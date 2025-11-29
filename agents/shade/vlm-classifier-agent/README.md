# VLM Classifier Agent

A specialized **Private Visual Language Model (VLM)** agent designed for automated content classification within a Trusted Execution Environment (TEE). It runs **Qwen3-VL-2B** locally on CPU, ensuring image data never leaves the secure enclave.

## Features
- **Private Local Inference:** Runs Qwen3-VL-2B-Instruct directly in the container (CPU). No external API calls for inference.
- **Context Aware:** Can load community guidelines from `data/community-{id}/` to inform classification.
- **CPU Optimized:** Uses `transformers` for reliable execution on Phala Cloud TEEs (Intel TDX), avoiding the complexity and GPU-dependencies of vLLM.
- **Visual Analysis:** Classifies images based on custom labels or open-ended prompts.
- **On-Chain Verification:** Logs classification results (hash + label) to the NEAR `shade-classifier-agent` contract.
- **Self-Contained:** Model weights are baked into the Docker image for complete isolation.

## API Reference

### `POST /classify`

**Request:**
```json
{
  "userId": "user-1",
  "communityId": "default",
  "imageUrl": "https://example.com/image.png",
  "labels": ["safe", "nsfw", "spam"]
}
```

**Response:**
```json
{
  "label": "safe",
  "confidence_bps": 9000,
  "sessionId": "sha256...",
  "raw_output": "The image depicts..."
}
```

## Configuration

| Variable | Description | Required |
|----------|-------------|----------|
| `USER_ID` | Authorized User ID (security binding) | Yes |
| `CLASSIFIER_CONTRACT_ID` | NEAR contract for logging | No (Optional) |
| `SHADE_AGENT_API_URL` | Local Shade sidecar URL (for local dev) | No (Default: http://localhost:3140/api/agent) |
| `DATA_DIR` | Path to community data | No (Default: data) |

## Development

**Note:** This agent requires Python 3.10+.

```bash
# Install dependencies
pip install -r requirements.txt

# Run dev server
python src/main.py
```

## Deployment

Build the production Docker image:
```bash
./scripts/deploy.sh
```

**⚠️ Important:** This Docker image will be large (~5GB) because it includes the Qwen3-VL-2B model weights. This is intentional to ensure the agent can run without external model dependencies.
