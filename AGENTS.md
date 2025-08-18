# GroupWeave Monorepo - Agent & Developer Instructions

This document provides comprehensive instructions for developers and AI agents...

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

## Coding Style & Linting
### Javascript / Typescript
- Formatted with Prettier, linted with ESLint. Follows a specific import order.

### Python
- Formatted with black and isort, linted with flake8 and mypy.

### Rust
- Formatted with rustfmt, linted with cargo clippy.
