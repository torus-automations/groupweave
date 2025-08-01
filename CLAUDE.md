# CLAUDE.md - GroupWeave Development Guide

This document provides essential information for Claude Code to effectively work with the GroupWeave codebase.

## Project Overview

GroupWeave is an open-source infrastructure for user-owned AI, designed to enable co-creation and co-immersion in generative AI content. The project is currently developing on NEAR blockchain with plans to integrate Shade agents as customizable, semi-autonomous assistants.

## Technology Stack

- **Frontend**: Next.js (React-based)
- **Language**: JavaScript with TypeScript support configured
- **Blockchain**: NEAR Protocol integration
- **Dependencies**: Shade Agent JS SDK, Ethers.js, Viem
- **Container**: Docker support available

## Development Setup

### Prerequisites
- Node.js (latest LTS recommended)
- Yarn package manager
- Docker (for containerized development)

### Installation
```bash
yarn install
```

### Development Server
```bash
yarn dev
# or
yarn start
```

## Available Scripts

| Command | Description |
|---------|-------------|
| `yarn dev` | Start development server |
| `yarn start` | Start development server (alias) |
| `yarn phala:test` | Test with Phala network |
| `yarn docker:test` | Build and run Docker container locally |
| `yarn docker:prune` | Clean up Docker system |
| `yarn docker:image` | Build production Docker image |
| `yarn docker:push` | Push Docker image to registry |

## Project Structure

```
/
├── components/          # React components
├── pages/              # Next.js pages
├── public/             # Static assets
├── styles/             # CSS modules and global styles
├── docker-compose.yaml # Docker configuration
├── next.config.js      # Next.js configuration
└── package.json        # Dependencies and scripts
```

## Development Guidelines

### Code Style
- Follow standard JavaScript/TypeScript conventions
- Use React functional components with hooks
- Utilize Next.js patterns for routing and SSR when appropriate

### Testing
- Test framework: AVA (configured in devDependencies)
- Run tests before committing changes
- Ensure Docker builds succeed

### Blockchain Integration
- Uses NEAR Protocol for blockchain functionality
- Shade Agent SDK for AI agent interactions
- Ethers.js and Viem for broader Web3 compatibility

## Key Dependencies

- `@neardefi/shade-agent-js`: Core SDK for Shade agents
- `chainsig.js`: Chain signature functionality
- `ethers` / `viem`: Ethereum/Web3 libraries
- `next`: React framework

## Environment Configuration

Create `.env.development.local` for local development with necessary environment variables for:
- NEAR network configuration
- API keys
- Blockchain RPC endpoints

## Docker Development

### Local Testing
```bash
yarn docker:test
```

### Production Build
```bash
yarn docker:image
```

## Future Expansion

As the project grows to include Python and Rust components:

### Python Components (Planned)
- Set up virtual environment: `python -m venv venv`
- Install dependencies: `pip install -r requirements.txt`
- Run tests: `pytest`
- Linting: `flake8` or `black`

### Rust Components (Planned)
- Build: `cargo build`
- Test: `cargo test`
- Linting: `cargo clippy`
- Formatting: `cargo fmt`

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly (including Docker builds)
5. Submit a pull request for review

## Notes for Claude Code

- Always run development server to test changes: `yarn dev`
- Verify Docker builds work: `yarn docker:test`
- Check for TypeScript errors in Next.js configuration
- Be mindful of blockchain-specific dependencies and their requirements
- Test environment variable requirements for NEAR/blockchain functionality