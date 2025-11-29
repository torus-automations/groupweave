# Shade Curation Agent

A production-grade **Private Retrieval Augmented Generation (RAG)** agent designed for Trusted Execution Environments (TEEs). It combines local document retrieval with a local LLM (**Phi-3-mini**) to answer community moderation questions without exposing private data to external APIs.

## Features
- **Private Local RAG:** Runs entirely within the container. No external LLM API calls.
- **Local Inference:** Uses **Microsoft Phi-3-mini-4k-instruct** on CPU.
- **On-Chain Logging:** Logs interaction hashes to the NEAR `shade-curation-agent` contract for auditability.
- **Community Isolation:** Enforces strict separation of data by `COMMUNITY_ID`.

## Security & Encryption (Confidential Context)

**Production Requirement:**
For production deployments on Phala Cloud, **End-to-End Encryption (E2EE)** is mandatory for confidential context updates.
- The calling application **must encrypt** the confidential context payload before sending it to the Shade Agent.
- The Shade Agent (running in a TEE) receives the encrypted payload, decrypts it securely using the TEE's private key, and updates the local context index.
- This ensures that sensitive community guidelines or private data are never exposed in transit or to the host infrastructure.

**Development/Testing:**
- For local development and testing environments, encryption can be disabled/bypassed. The agent will accept plain text context updates for ease of debugging.

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
| `SHADE_AGENT_API_URL` | Local Shade sidecar URL (for local dev) | No (Default: http://localhost:3140/api/agent) |

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