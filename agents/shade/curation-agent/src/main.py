import os
import glob
import hashlib
import logging
import requests
import uuid
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import List, Optional, Dict
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer, pipeline
from sentence_transformers import SentenceTransformer
import faiss
import numpy as np

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
    # Fallback or exit? For production, we exit.
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
            "cost_microusd": 0, # Local compute is "free" (sunk cost)
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

    # RAG Retrieval
    context = retrieve(user_query)
    
    # Construct Prompt (Phi-3 Instruct format)
    # <|user|>
    # {prompt} <|end|>
    # <|assistant|>
    
    system_prompt = "You are a helpful curation assistant. Use the Context to answer."
    full_prompt = f"<|system|>
{system_prompt}
Context:
{context}<|end|>
"
    
    for m in req.messages:
        full_prompt += f"<|{m.role}|>
{m.content}<|end|>
"
    
    full_prompt += "<|assistant|>
"

    # Generation
    outputs = llm_pipeline(full_prompt)
    answer = outputs[0]["generated_text"].split("<|assistant|>
")[-1].strip()

    # Logging
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
    return {"status": "ok", "docs_indexed": len(docs)}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=PORT)
