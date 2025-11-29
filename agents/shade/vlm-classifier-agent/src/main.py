import os
import json
import hashlib
import logging
import requests
import socket
import ipaddress
import uuid
import glob
from urllib.parse import urlparse
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import List, Optional
from transformers import Qwen2VLForConditionalGeneration, AutoProcessor
from qwen_vl_utils import process_vision_info
import torch

# Configuration
PORT = int(os.getenv("PORT", 3001))
SHADE_AGENT_API_URL = os.getenv("SHADE_AGENT_API_URL", "http://localhost:3140/api/agent")
CLASSIFIER_CONTRACT_ID = os.getenv("CLASSIFIER_CONTRACT_ID", "")
USER_ID = os.getenv("USER_ID", "").strip()
DATA_DIR = os.getenv("DATA_DIR", "data")

# Logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize FastAPI
app = FastAPI()

# Load Model (Global)
logger.info("Loading Qwen3-VL-2B-Instruct (CPU)...")
try:
    model = Qwen2VLForConditionalGeneration.from_pretrained(
        "Qwen/Qwen3-VL-2B-Instruct",
        torch_dtype=torch.float32, # CPU supports float32 best
        device_map="cpu"
    )
    processor = AutoProcessor.from_pretrained("Qwen/Qwen3-VL-2B-Instruct")
    logger.info("Model loaded successfully.")
except Exception as e:
    logger.error(f"Failed to load model: {e}")
    raise e

class ClassifyRequest(BaseModel):
    userId: str
    imageUrl: str
    communityId: Optional[str] = None
    labels: Optional[List[str]] = None
    prompt: Optional[str] = None

def sha256_hex(s: str) -> str:
    return hashlib.sha256(s.encode()).hexdigest()

def load_context(community_id: str) -> str:
    target_dir = os.path.join(DATA_DIR, f"community-{community_id}")
    if not os.path.exists(target_dir):
        return ""
    
    context_parts = []
    files = glob.glob(os.path.join(target_dir, "*.*"))
    for f in files:
        if f.endswith(".txt") or f.endswith(".md"):
            try:
                with open(f, "r", encoding="utf-8") as file:
                    context_parts.append(file.read())
            except Exception as e:
                logger.warning(f"Failed to read context file {f}: {e}")
    
    return "\n\n".join(context_parts)

def shade_log_classification(session_id: str, image_hash: str, prompt_hash: str, label: str, confidence_bps: int):
    if not CLASSIFIER_CONTRACT_ID:
        logger.warning("CLASSIFIER_CONTRACT_ID not set; skipping on-chain log")
        return

    payload = {
        "contractId": CLASSIFIER_CONTRACT_ID,
        "methodName": "log_classification",
        "args": {
            "session_id": session_id,
            "image_hash": image_hash,
            "prompt_hash": prompt_hash,
            "label": label,
            "confidence_bps": confidence_bps,
            "model": "qwen3-vl-2b-cpu",
        },
        "gas": "100000000000000",
        "attachedDeposit": "10000000000000000000000"
    }

    try:
        # Note: The sidecar expects { method: "functionCall", ...params }
        # We need to match the JS agent('functionCall', ...) signature logic
        # Usually that maps to: POST /api/agent with body { method: 'functionCall', ...payload }
        full_payload = {
            "method": "functionCall",
            **payload
        }
        resp = requests.post(SHADE_AGENT_API_URL, json=full_payload, timeout=5)
        resp.raise_for_status()
        logger.info(f"Logged to chain: {resp.text}")
    except Exception as e:
        logger.error(f"Failed to log to chain: {e}")

def validate_url(url: str):
    try:
        parsed = urlparse(url)
        if not parsed.hostname:
            raise ValueError("Invalid URL")
        
        # Resolve hostname to IP
        ip = socket.gethostbyname(parsed.hostname)
        ip_addr = ipaddress.ip_address(ip)
        
        if ip_addr.is_private or ip_addr.is_loopback or ip_addr.is_multicast:
            raise ValueError(f"Access to private IP {ip} is denied")
            
    except Exception as e:
        logger.warning(f"URL Validation failed for {url}: {e}")
        raise HTTPException(status_code=400, detail="Invalid or restricted Image URL")

@app.post("/classify")
async def classify(req: ClassifyRequest):
    if USER_ID and req.userId != USER_ID:
        raise HTTPException(status_code=403, detail="User mismatch")

    validate_url(req.imageUrl)

    session_id = str(uuid.uuid4())

    # Load Context
    context = ""
    if req.communityId:
        context = load_context(req.communityId)

    # Construct Prompt
    base_prompt = req.prompt or "Describe this image."
    
    if context:
        text_prompt = f"Context:\n{context}\n\nTask: {base_prompt}"
    else:
        text_prompt = base_prompt

    if req.labels:
        text_prompt = f"Classify this image into one of: {', '.join(req.labels)}. {text_prompt}"

    messages = [
        {
            "role": "user",
            "content": [
                {"type": "image", "image": req.imageUrl},
                {"type": "text", "text": text_prompt},
            ],
        }
    ]

    # Inference
    try:
        text = processor.apply_chat_template(messages, tokenize=False, add_generation_prompt=True)
        image_inputs, video_inputs = process_vision_info(messages)
        inputs = processor(
            text=[text],
            images=image_inputs,
            videos=video_inputs,
            padding=True,
            return_tensors="pt",
        )
        
        # Move inputs to CPU
        inputs = inputs.to("cpu")

        generated_ids = model.generate(**inputs, max_new_tokens=128)
        generated_ids_trimmed = [
            out_ids[len(in_ids) :] for in_ids, out_ids in zip(inputs.input_ids, generated_ids)
        ]
        output_text = processor.batch_decode(
            generated_ids_trimmed, skip_special_tokens=True, clean_up_tokenization_spaces=False
        )[0]

        # Basic parsing (naive)
        label = output_text.strip()
        
        confidence_bps = 0
        # If specific labels were requested, try to match
        if req.labels:
            for l in req.labels:
                if l.lower() in label.lower():
                    label = l
                    confidence_bps = 10000 # High confidence if keyword matched
                    break
        else:
            # Open-ended description, confidence is undefined/implicit
            confidence_bps = 0

        # Log to chain
        image_hash = sha256_hex(req.imageUrl)
        prompt_hash = sha256_hex(text_prompt)
        shade_log_classification(session_id, image_hash, prompt_hash, label, confidence_bps)

        return {
            "label": label,
            "confidence_bps": confidence_bps,
            "sessionId": session_id,
            "raw_output": output_text
        }

    except Exception as e:
        logger.error(f"Inference failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/health")
def health():
    return {"status": "ok", "model": "Qwen3-VL-2B-Instruct"}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=PORT)
