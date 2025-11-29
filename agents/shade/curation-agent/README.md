# Shade Curation Agent

A production-grade **Private Retrieval Augmented Generation (RAG)** agent designed for Trusted Execution Environments (TEEs). It combines local document retrieval with a local LLM (**Phi-3-mini**) to answer community moderation questions without exposing private data to external APIs.

## Features
- **Private Local RAG:** Runs entirely within the container. No external LLM API calls.
- **Local Inference:** Uses **Microsoft Phi-3-mini-4k-instruct** on CPU.
- **On-Chain Logging:** Logs interaction hashes to the NEAR `shade-curation-agent` contract for auditability.
- **Community Isolation:** Enforces strict separation of data by `COMMUNITY_ID`.

## API Reference

### `POST /chat`

**Request:**
```json
{
  "messages": [
    { "role": "user", "content": "Is AI art allowed?" }
  ],
  "communityId": "dw"
}
```

**Response:**
```json
{
  "answer": "Yes, but it must be tagged.",
  "sessionId": "uuid-v4...",
  "queryHash": "sha256...",
  "answerHash": "sha256..."
}
```

## Configuration

| Variable | Description | Required |
|----------|-------------|----------|
| `COMMUNITY_ID` | Identifier for the community (e.g., `dw`) | Yes |
| `CURATION_CONTRACT_ID` | NEAR contract for logging | No (Optional) |
| `SHADE_AGENT_API_URL` | Local Shade sidecar URL | No (Default: localhost:3140) |

## Development

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

**⚠️ Important:** This Docker image will be large (~3-4GB) as it includes the Phi-3 and embedding models.