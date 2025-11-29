import os
import glob
import hashlib
import logging
import requests
import uuid
import json
import re
from contextlib import AsyncExitStack
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import List, Optional, Dict, Any
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer, pipeline
from sentence_transformers import SentenceTransformer
import faiss
import numpy as np

# MCP Imports
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

# Configuration
PORT = int(os.getenv("PORT", 3000))
SHADE_AGENT_API_URL = os.getenv("SHADE_AGENT_API_URL", "http://localhost:3140/api/agent")
CURATION_CONTRACT_ID = os.getenv("CURATION_CONTRACT_ID", "")
COMMUNITY_ID = os.getenv("COMMUNITY_ID", "").strip()
DATA_DIR = os.getenv("DATA_DIR", "data")

# Logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI()

# Global State
index = None
docs: List[Dict] = []
llm_pipeline = None
mcp_session: Optional[ClientSession] = None
mcp_exit_stack: Optional[AsyncExitStack] = None

# Load Models
logger.info("Loading Embedding Model (all-MiniLM-L6-v2)...")
embedder = SentenceTransformer('all-MiniLM-L6-v2')

logger.info("Loading Local LLM (Phi-3-mini-4k-instruct)...")
try:
    model = AutoModelForCausalLM.from_pretrained(
        "microsoft/Phi-3-mini-4k-instruct",
        torch_dtype=torch.float32,
        device_map="cpu",
        trust_remote_code=True
    )
    tokenizer = AutoTokenizer.from_pretrained("microsoft/Phi-3-mini-4k-instruct", trust_remote_code=True)
    
    llm_pipeline = pipeline(
        "text-generation",
        model=model,
        tokenizer=tokenizer,
        max_new_tokens=512,
        return_full_text=False,
        do_sample=True,
        temperature=0.1,
    )
    logger.info("Models loaded successfully.")
except Exception as e:
    logger.error(f"Failed to load LLM: {e}")
    raise e

# In-Memory RAG Indexing
def build_index():
    global index, docs
    docs = []
    
    target_dir = os.path.join(DATA_DIR, f"community-{COMMUNITY_ID}") if COMMUNITY_ID else DATA_DIR
    if not os.path.exists(target_dir):
        logger.warning(f"Data directory not found: {target_dir}")
        return

    files = glob.glob(os.path.join(target_dir, "*.*"))
    corpus = []
    
    for f in files:
        if f.endswith(".md") or f.endswith(".txt"):
            with open(f, "r", encoding="utf-8") as file:
                content = file.read()
                docs.append({"id": os.path.basename(f), "text": content})
                corpus.append(content)
    
    if not corpus:
        logger.warning("No documents found to index.")
        return

    logger.info(f"Indexing {len(corpus)} documents...")
    embeddings = embedder.encode(corpus)
    dimension = embeddings.shape[1]
    index = faiss.IndexFlatL2(dimension)
    index.add(np.array(embeddings))
    logger.info("Indexing complete.")

build_index()

# MCP Lifecycle
@app.on_event("startup")
async def startup_event():
    global mcp_session, mcp_exit_stack
    
    # Start local search server
    # Assuming src/search_server.py exists and python is available
    server_script = os.path.join(os.path.dirname(__file__), "search_server.py")
    
    if not os.path.exists(server_script):
        logger.warning(f"Search server script not found at {server_script}")
        return

    server_params = StdioServerParameters(
        command="python", 
        args=[server_script],
        env=None
    )
    
    mcp_exit_stack = AsyncExitStack()
    
    try:
        read, write = await mcp_exit_stack.enter_async_context(stdio_client(server_params))
        mcp_session = await mcp_exit_stack.enter_async_context(ClientSession(read, write))
        await mcp_session.initialize()
        logger.info("Connected to MCP Search Server")
    except Exception as e:
        logger.error(f"Failed to connect to MCP: {e}")

@app.on_event("shutdown")
async def shutdown_event():
    if mcp_exit_stack:
        await mcp_exit_stack.aclose()

class ChatMessage(BaseModel):
    role: str
    content: str

class ChatRequest(BaseModel):
    messages: List[ChatMessage]
    communityId: Optional[str] = None

def sha256_hex(s: str) -> str:
    return hashlib.sha256(s.encode()).hexdigest()

def retrieve(query: str, k=3) -> str:
    if not index or not docs:
        return ""
    
    q_embed = embedder.encode([query])
    D, I = index.search(np.array(q_embed), k)
    
    results = []
    for idx in I[0]:
        if idx != -1 and idx < len(docs):
            results.append(docs[idx]['text'])
            
    return "\n\n---\n\n".join(results)

def shade_log_interaction(session_id: str, query_hash: str, answer_hash: str):
    if not CURATION_CONTRACT_ID:
        return

    payload = {
        "contractId": CURATION_CONTRACT_ID,
        "methodName": "log_interaction",
        "args": {
            "session_id": session_id,
            "query_hash": query_hash,
            "answer_hash": answer_hash,
            "cost_microusd": 0, # Local compute is "free"
            "community_id": COMMUNITY_ID
        },
        "gas": "100000000000000",
        "attachedDeposit": "10000000000000000000000"
    }

    try:
        full_payload = { "method": "functionCall", **payload }
        requests.post(SHADE_AGENT_API_URL, json=full_payload, timeout=5)
    except Exception as e:
        logger.error(f"Chain log failed: {e}")

@app.post("/chat")
async def chat(req: ChatRequest):
    if COMMUNITY_ID and req.communityId and req.communityId != COMMUNITY_ID:
        raise HTTPException(status_code=403, detail="Community mismatch")

    user_query = next((m.content for m in reversed(req.messages) if m.role == "user"), "")
    if not user_query:
        raise HTTPException(status_code=400, detail="No user message found")

    # 1. Retrieve Local Context
    rag_context = retrieve(user_query)
    
    # 2. Prepare Tools (MCP)
    available_tools = []
    if mcp_session:
        try:
            tools_result = await mcp_session.list_tools()
            available_tools = tools_result.tools
        except Exception as e:
            logger.warning(f"MCP list_tools failed: {e}")

    tool_descriptions = ""
    if available_tools:
        descriptions = [f"- {t.name}: {t.description}" for t in available_tools]
        tool_descriptions = "Available Tools:\n" + "\n".join(descriptions) + "\n\nTo use a tool, reply STRICTLY in this format: [TOOL:<tool_name>|<args_json>]"

    # 3. Construct System Prompt
    system_prompt = "You are a helpful curation assistant. Use the Context to answer."
    if tool_descriptions:
        system_prompt += f"\n\n{tool_descriptions}\nIf you need external information, use the search tool."

    full_prompt = f"<|system|>\n{system_prompt}\nContext:\n{rag_context}<|end|>\n"
    
    for m in req.messages:
        full_prompt += f"<|{m.role}|>\n{m.content}<|end|>\n"
    
    full_prompt += "<|assistant|>\n"

    # 4. First Generation
    outputs = llm_pipeline(full_prompt)
    answer = outputs[0]["generated_text"].split("<|assistant|>\n")[-1].strip()

    # 5. Handle Tool Use Loop (Single Step)
    tool_match = re.search(r"\\[TOOL:(\w+)\|({.*?})\\]", answer)
    if tool_match and mcp_session:
        tool_name = tool_match.group(1)
        tool_args_str = tool_match.group(2)
        
        logger.info(f"Tool execution requested: {tool_name} {tool_args_str}")
        
        tool_result = "Tool execution failed."
        try:
            tool_args = json.loads(tool_args_str)
            result = await mcp_session.call_tool(tool_name, arguments=tool_args)
            # Accessing result.content (list of TextContent or ImageContent)
            tool_result = "\n".join([c.text for c in result.content if hasattr(c, "text")])
        except Exception as e:
            tool_result = f"Error: {str(e)}"
            logger.error(tool_result)

        # Feed back to LLM
        follow_up_prompt = f"{full_prompt}{answer}\n<|tool|>\n{tool_result}<|end|>\n<|assistant|>\n"
        
        outputs_2 = llm_pipeline(follow_up_prompt)
        answer = outputs_2[0]["generated_text"].split("<|tool|>\n")[-1].split("<|assistant|>\n")[-1].strip()

    # 6. Logging
    session_id = str(uuid.uuid4())
    shade_log_interaction(session_id, sha256_hex(user_query), sha256_hex(answer))

    return {
        "answer": answer,
        "sessionId": session_id,
        "queryHash": sha256_hex(user_query),
        "answerHash": sha256_hex(answer)
    }

@app.get("/health")
def health():
    return {"status": "ok", "docs_indexed": len(docs), "mcp_connected": mcp_session is not None}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=PORT)