# GroupWeave Monorepo - Agent & Developer Instructions

This document provides comprehensive instructions for developers and AI agents working on this monorepo. It is the single source of truth for repository setup, development workflows, and coding standards. Please read it carefully before contributing.

## Overview
GroupWeave is the open-source infrastructure for user-owned AI, designed to enable co-creation and co-immersion in generative AI content. Currently developing on NEAR, with plans to integrate Shade agents as customizable, semi-autonomous assistants for multimodal content understanding, moderation, and curation.

## Monorepo Tooling
### pnpm
Used for high-performance package management. All npm/yarn commands should be replaced with pnpm.

### Turborepo
Used to manage tasks and build pipelines across the monorepo, enabling faster builds and development.

## Global Commands
- `pnpm install`: Install all dependencies for all workspaces.
- `pnpm build`: Builds all apps and packages.
- `pnpm dev`: Runs all applications in development mode. Use with --filter to run a specific app.
- `pnpm lint`: Lints all apps and packages.
- `pnpm check-types`: Runs TypeScript type checking across all relevant packages.

## Workspace Details
- **`apps/creation`**: A Next.js application for creating new GroupWeave content.
- **`apps/dashboard`**: A Next.js application for users to view and manage their content and participation.
- **`apps/docs`**: A Next.js application for viewing project documentation.
- **`apps/participation`**: A Next.js application for participating in GroupWeave experiences and rounds.
- **`apps/mobile`**: A React Native application for GroupWeave on mobile, built with Expo.
- **`apps/api`**: A FastAPI backend providing the main API for GroupWeave services.
- **`apps/agents`**: Contains multiple Rust-based AI agent binaries for various automated tasks.
- **`apps/contracts`**: A workspace containing NEAR smart contracts for on-chain logic, including staking and voting.
- **`packages/ui`**: A shared React component library used across the frontend applications.
- **`packages/common-types`**: Shared TypeScript types and interfaces for consistency across the monorepo.
- **`packages/near`**: Shared utilities and configuration for interacting with the NEAR blockchain.
- **`packages/eslint-config`**: Shared ESLint configurations for maintaining code quality.
- **`packages/tailwind-config`**: Shared Tailwind CSS configuration for consistent styling.
- **`packages/typescript-config`**: Shared TypeScript configurations (tsconfig) for the monorepo.

## Coding Style & Linting
### Javascript / Typescript
- Formatted with Prettier, linted with ESLint. Follows a specific import order. See `packages/eslint-config`.

### Python
- Formatted with black and isort, linted with flake8 and mypy. Configuration is in the `apps/api` directory.

### Rust
- Formatted with rustfmt, linted with cargo clippy. Configuration is in the respective Cargo.toml files.

