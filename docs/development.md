# GroupWeave Monorepo - Agent & Developer Instructions

This document provides comprehensive instructions for developers and AI agents working within the GroupWeave monorepo. This project is a complex system with multiple applications and services. Adhering to these guidelines will ensure consistency, correctness, and a smooth development experience.

## Table of Contents

1.  [Overview](#overview)
2.  [Monorepo Tooling](#monorepo-tooling)
    -   [pnpm](#pnpm)
    -   [Turborepo](#turborepo)
    -   [Remote Caching](#remote-caching)
3.  [Global Commands](#global-commands)
4.  [Development Preferences](#development-preferences)
5.  [Workspace Details](#workspace-details)
    -   [Frontend Apps (`apps/web`, `apps/docs`)](#frontend-apps)
    -   [Mobile App (`apps/mobile`)](#mobile-app)
    -   [Python API (`apps/api`)](#python-api)
    -   [Rust Projects (`apps/agents`, `apps/contracts`)](#rust-projects)
    -   [Shared Packages (`packages/*`)](#shared-packages)
6.  [Key Files](#key-files)
7.  [Dependencies](#dependencies)
8.  [Coding Style & Linting](#coding-style--linting)
    -   [JavaScript/TypeScript](#javascripttypescript)
    -   [Python](#python)
    -   [Rust](#rust)

## Overview

GroupWeave is an open-source platform for user-owned AI, focusing on co-creation and co-immersion in generative content. This monorepo contains all the code for the web application, documentation site, mobile app, backend API, smart contracts, and AI agents.

## Monorepo Tooling

This repository is a monorepo managed by **pnpm** and **Turborepo**.

### pnpm
Used for package management. All `npm`/`yarn` commands should be replaced with `pnpm`.

### Turborepo
Used to manage tasks and build pipelines across the monorepo. You can run commands from the root, and Turborepo will execute them in the relevant workspaces.

### Remote Caching

Turborepo can use a technique known as [Remote Caching](https://turborepo.com/docs/core-concepts/remote-caching) to share cache artifacts across machines, enabling you to share build caches with your team and CI/CD pipelines.

To enable Remote Caching, you will need a Vercel account.
1.  **Login to Vercel**:
    ```bash
    pnpm exec turbo login
    ```
2.  **Link the repository**:
    ```bash
    pnpm exec turbo link
    ```

## Global Commands

You can run these commands from the root of the repository.

-   `pnpm install`: Install all dependencies for all workspaces.
-   `pnpm build`: Builds all apps and packages.
-   `pnpm dev`: Runs all applications in development mode. (Note: This is resource-intensive. It's better to run a specific app's dev server using filters).
-   `pnpm lint`: Lints all apps and packages.
-   `pnpm check-types`: Runs TypeScript type checking across all relevant packages.
-   `pnpm format`: Formats all supported files with Prettier.

You can also use filters to run a command in a specific workspace, for example: `pnpm --filter web dev`.

## Development Preferences

-   Always run `pnpm lint` and `pnpm check-types` before committing to maintain a zero-warnings policy.
-   Follow existing component patterns in `@repo/ui`.
-   Test mobile changes across platforms when possible.
-   Coordinate changes that span across multiple workspace dependencies.

## Workspace Details

### Frontend Apps (`apps/web`, `apps/docs`)

Next.js applications written in TypeScript.

-   **`apps/web`**: The main GroupWeave web application.
-   **`apps/docs`**: The documentation website.

**To run a specific frontend app:**
```bash
# To run the web app (available at http://localhost:3002)
pnpm --filter web dev

# To run the docs site (available at http://localhost:3001)
pnpm --filter docs dev
```

### Mobile App (`apps/mobile`)

A React Native application built with **Expo**.

**To run the mobile app:**
First, ensure you have the Expo CLI installed (`pnpm install -g expo-cli`) and meet the requirements for React Native development.

```bash
# Navigate to the mobile app directory
cd apps/mobile

# Start the development server
pnpm start
```
Then, use the Expo Dev Tools in your browser to run on a simulator or device.

### Python API (`apps/api`)

A FastAPI backend.

**Setup:**
1.  `cd apps/api`
2.  `python -m venv .venv`
3.  `source .venv/bin/activate`
4.  `pip install -r requirements.txt`
5.  Create a `.env` file. Requires `GEMINI_API_KEY` and `FAL_API_KEY`.

**Running the server:**
```bash
# from apps/api with venv activated
uvicorn main:app --reload --port 8000
```

### Rust Projects

Two main Rust projects: `apps/agents` and `apps/contracts`. Requires the Rust toolchain (`rustup`).

#### AI Agents (`apps/agents`)
Contains multiple agent binaries. See the `apps/agents/README.md` for specific instructions on the Shade Agent.
-   **Commands (from `apps/agents`):** `cargo check`, `cargo build`, `cargo test`, `cargo clippy`.

#### Smart Contracts (`apps/contracts`)
A workspace containing NEAR smart contracts. See `apps/contracts/README.md` for details.
-   **Commands (from a contract dir):** `cargo check`, `cargo test`, `cargo clippy`.
-   **Build WASM:** `cargo build --target wasm32-unknown-unknown --release`

### Shared Packages (`packages/*`)

-   **`@repo/ui`**: Shared React component library.
-   **`@groupweave/common-types`**: Shared TypeScript types.
-   **`@repo/eslint-config`**, **`@repo/tailwind-config`**, **`@repo/typescript-config`**: Shared configurations.

## Key Files
-   `/turbo.json`: Build pipeline configuration.
-   `/pnpm-workspace.yaml`: Defines the workspaces in the monorepo.
-   `/apps/web/components/`: Main UI components for the web app.
-   `/packages/ui/`: The shared component library.
-   `/apps/api/requirements.txt`: Python dependencies.
-   `/apps/*/Cargo.toml`: Rust project configurations.

## Dependencies

To install all dependencies for the monorepo, run this command from the root directory:
```bash
pnpm install
```

## Coding Style & Linting

### JavaScript/TypeScript
-   **Formatting**: Run `pnpm format` to format code with Prettier.
-   **Linting**: Run `pnpm lint` to check for code quality issues.
-   **Import Style**: Always use `import type` when importing only types.
    -   **Good**: `import type { SomeType } from './types';`
    -   **Bad**: `import { SomeType } from './types';`
-   **Import Order**:
    1.  React imports
    2.  External library imports
    3.  Internal absolute path imports (e.g., from `@repo/ui`)
    4.  Relative path imports

### Python
-   **Formatting**: Use `black` and `isort`.
-   **Linting**: Use `flake8` and `mypy`.

### Rust
-   **Formatting**: Use `rustfmt` (usually via `cargo fmt`).
-   **Linting**: Use `cargo clippy`.
