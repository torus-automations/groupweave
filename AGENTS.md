# GroupWeave Monorepo - Agent & Developer Instructions

This document provides comprehensive instructions for developers and AI agents working on this monorepo.

## Overview
An open-source platform for user-owned AI, focusing on co-creation and co-immersion in generative content.

## Monorepo Tooling
### pnpm
Used for package management. All npm/yarn commands should be replaced with pnpm.

### Turborepo
Used to manage tasks and build pipelines across the monorepo.

## Global Commands
- `pnpm install`: Install all dependencies for all workspaces.
- `pnpm build`: Builds all apps and packages.
- `pnpm dev`: Runs all applications in development mode.
- `pnpm lint`: Lints all apps and packages.

## Workspace Details
- **`apps/web`**: The main GroupWeave web application (Next.js).
- **`apps/docs`**: The documentation website (Next.js).
- **`apps/mobile`**: A React Native application built with Expo.
- **`apps/api`**: A FastAPI backend.
- **`apps/agents`**: Contains multiple Rust-based AI agent binaries.
- **`apps/contracts`**: A workspace containing NEAR smart contracts.
- **`packages/ui`**: Shared React component library.
- **`packages/common-types`**: Shared TypeScript types.

## Dependency Management
**⚠️ IMPORTANT: All dependency updates are handled by Renovate. Do not manually update dependencies.**

This repository uses [Renovate](https://docs.renovatebot.com/) to automatically manage dependency updates across all languages and frameworks. Renovate creates PRs for dependency updates that should be reviewed and merged by maintainers.

### What Renovate Manages
- **JavaScript/TypeScript**: npm packages in all `package.json` files
- **Python**: Dependencies in `requirements.txt` files
- **Rust**: Crates in `Cargo.toml` files
- **Docker**: Base images in `Dockerfile` and `docker-compose.yaml`
- **GitHub Actions**: Action versions in workflow files

### For Contributors & AI Agents
- ✅ **DO**: Add new dependencies as needed for features (`pnpm add`, `pip install`, `cargo add`)
- ❌ **DON'T**: Update existing dependency versions manually
- ❌ **DON'T**: Modify `pnpm-lock.yaml`, lockfiles, or version ranges unless adding new deps
- 🔍 **REVIEW**: Test and approve Renovate PRs when they appear

Manual dependency updates will conflict with Renovate and should be avoided. If you encounter urgent security issues requiring immediate dependency updates, create an issue and tag maintainers.

## Coding Style & Linting
### Javascript / Typescript
- Formatted with Prettier, linted with ESLint. Follows a specific import order.

### Python
- Formatted with black and isort, linted with flake8 and mypy.

### Rust
- Formatted with rustfmt, linted with cargo clippy.

