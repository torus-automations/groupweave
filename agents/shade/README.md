# Shade Agents

This directory contains the autonomous agents built on the Shade protocol (Phala Network TEEs).

## Architecture

- **`curation-agent/`**: A TEE-ready agent that provides Retrieval Augmented Generation (RAG) over private community documents. It coordinates with the `shade-curation-agent` smart contract on NEAR.
- **`vlm-classifier-agent/`**: A user-owned agent using OpenAI's GPT-4o-mini (Vision) to classify images. It logs results on-chain via the `shade-classifier-agent` contract.

## Usage

Each agent is a standalone containerized application.

### Development
Navigate to an agent directory and run:
```bash
bun install
bun run dev
```

### Local Simulation (Docker Compose)
To test the agent with the Shade sidecar (mocking the TEE environment):
```bash
docker-compose up --build
```
This will start the agent (`app`) and the `shade-agent-api` service. Ensure your `.env.development.local` is configured.

### Deployment
Each agent includes a deployment script to build its Docker image:
```bash
./scripts/deploy.sh
```
This will build the image `groupweave/<agent-name>:latest`. You can then push this image to your registry and register it with the Shade Protocol.

## Prerequisites
- Node.js >= 20
- Bun (recommended) or npm
- Docker (for deployment)