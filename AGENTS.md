# AGENTS.md

This file provides comprehensive instructions for AI agents working on the GroupWeave monorepo.

## Project Overview

GroupWeave is an open-source infrastructure for user-owned AI, designed to enable co-creation and co-immersion in generative AI content. Currently developing on NEAR, with plans to integrate Shade agents as customizable, semi-autonomous assistants for multimodal content understanding, moderation, and curation.

## Repository Structure

This is a monorepo managed with **Turborepo** and **pnpm workspaces**.

### Applications (`apps/`)

-   **`apps/creation`**: Next.js app for creating new GroupWeave content
-   **`apps/dashboard`**: Next.js app for users to view and manage content/participation  
-   **`apps/docs`**: Next.js app for project documentation
-   **`apps/participation`**: Next.js app for participating in GroupWeave experiences
-   **`apps/mobile`**: React Native app built with Expo for mobile platforms
-   **`apps/api`**: FastAPI backend providing main API services
-   **`apps/agents`**: Rust-based AI agent binaries for automated tasks
-   **`apps/contracts`**: NEAR smart contracts (staking, voting, zkp-verifier)

### Shared Packages (`packages/`)

-   **`packages/ui`**: Shared React component library (shadcn/ui-inspired)
-   **`packages/common-types`**: TypeScript types and interfaces
-   **`packages/near`**: NEAR blockchain utilities and configuration
-   **`packages/eslint-config`**: Shared ESLint configurations
-   **`packages/tailwind-config`**: Shared Tailwind CSS configuration  
-   **`packages/typescript-config`**: Shared TypeScript configurations

---

## Quick Start

### Prerequisites
- Node.js >=18
- pnpm 9.0.0+
- Python 3.9+
- Rust 1.70+

### Essential Commands

```bash
# Install all dependencies
pnpm install

# Build all packages and apps  
pnpm build

# Run development servers for all apps
pnpm dev

# Lint all code
pnpm lint

# Type check all TypeScript
pnpm check-types

# Format all code
pnpm format
```

---

## Development Workflows

### Frontend Development

Run a specific Next.js app:
```bash
# Development server
pnpm --filter creation dev
pnpm --filter dashboard dev  
pnpm --filter docs dev
pnpm --filter participation dev
```

### Mobile Development

```bash
cd apps/mobile

# Start development server
pnpm start

# Run on specific platforms  
pnpm ios
pnpm android
```

### API Development

```bash
cd apps/api

# Setup Python environment
python -m venv .venv
source .venv/bin/activate  # Linux/macOS
# .venv\Scripts\activate   # Windows

# Install dependencies
pip install -r requirements.txt

# Run development server
uvicorn main:app --reload --port 8000
```

### Rust Development

```bash
cd apps/agents  # or apps/contracts

# Check code without building
cargo check

# Build project
cargo build

# Run tests
cargo test

# Lint code
cargo clippy

# Format code  
cargo fmt
```

---

## Testing

### Frontend Tests
```bash
# Run tests for specific app
pnpm --filter creation test
pnpm --filter dashboard test

# Run all frontend tests
pnpm test
```

### API Tests
```bash
cd apps/api
pytest
```

### Rust Tests
```bash
cd apps/agents  # or apps/contracts
cargo test
```

### Pre-commit Checklist
- [ ] `pnpm lint` passes
- [ ] `pnpm check-types` passes  
- [ ] All tests pass
- [ ] Code is formatted (`pnpm format`)

---

## Pull Request Guidelines

### Title Format
Use conventional commits: `type(scope): description`

**Examples:**
- `feat(creation): add image upload component`
- `fix(api): resolve authentication bug`  
- `docs(agents): update setup instructions`
- `refactor(ui): improve button component API`

### Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding/updating tests
- `chore`: Maintenance tasks

### PR Checklist
- [ ] Title follows conventional commit format
- [ ] All tests pass locally
- [ ] Code is linted and type-checked
- [ ] Documentation updated if needed
- [ ] Breaking changes are documented

---

## Code Style & Standards

### TypeScript/JavaScript
- **Formatter**: Prettier
- **Linter**: ESLint with custom configs
- **Import order**: Enforced via ESLint
- **Components**: Follow React best practices

### Python  
- **Formatter**: `black`
- **Import sorter**: `isort`
- **Linter**: `flake8`
- **Type checker**: `mypy`

### Rust
- **Formatter**: `rustfmt`
- **Linter**: `clippy`
- **Testing**: Standard `cargo test`

---

## Architecture Guidelines

### Shared UI Components
- Use `packages/ui` for reusable components
- Follow shadcn/ui patterns with Radix UI primitives
- Tailwind CSS for styling
- Components should be framework-agnostic where possible

### Type Safety
- Use `packages/common-types` for shared TypeScript interfaces
- Maintain strict type checking across all projects
- Document complex types with TSDoc comments

### NEAR Integration  
- Use `packages/near` for blockchain interactions
- Follow NEAR development best practices
- Smart contracts in `apps/contracts`

---

## Troubleshooting

### Common Issues

**Dependencies not resolving:**
```bash
pnpm install --frozen-lockfile=false
```

**Build failures:**
```bash
pnpm clean  # If available
pnpm build --force
```

**Type errors after changes:**
```bash
pnpm check-types --force
```

### Getting Help
- Check existing issues and documentation
- Review similar implementations in the codebase
- Ensure you're using the correct package manager (pnpm, not npm/yarn)

---

## Important Notes

1. **Package Manager**: Always use `pnpm`, never `npm` or `yarn`
2. **Node Version**: Ensure Node.js >=18 for compatibility  
3. **Documentation**: This repository uses automated doc generation via `generate_docs.py`
4. **Security**: Never commit secrets, API keys, or sensitive data
5. **Mobile**: React Native components are separate from web UI components
