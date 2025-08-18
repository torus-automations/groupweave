# GroupWeave Monorepo - Agent Instructions

Welcome, agent! This document provides instructions for working within the GroupWeave monorepo. This project is a complex system with multiple applications and services. Adhering to these guidelines will ensure consistency and correctness.

## Table of Contents

1.  [Overview](#overview)
2.  [Monorepo Tooling](#monorepo-tooling)
3.  [Global Commands](#global-commands)
4.  [Workspace Details](#workspace-details)
    -   [Frontend Apps (`apps/web`, `apps/docs`)](#frontend-apps)
    -   [Mobile App (`apps/mobile`)](#mobile-app)
    -   [Python API (`apps/api`)](#python-api)
    -   [Rust Projects (`apps/agents`, `apps/contracts`)](#rust-projects)
    -   [Shared Packages (`packages/*`)](#shared-packages)
5.  [Dependencies](#dependencies)
6.  [Coding Style & Linting](#coding-style--linting)

## Overview

GroupWeave is an open-source platform for user-owned AI, focusing on co-creation and co-immersion in generative content. This monorepo contains all the code for the web application, documentation site, mobile app, backend API, smart contracts, and AI agents.

## Monorepo Tooling

This repository is a monorepo managed by **pnpm** and **Turborepo**.

-   **pnpm**: Used for package management. All `npm`/`yarn` commands should be replaced with `pnpm`.
-   **Turborepo**: Used to manage tasks and build pipelines across the monorepo.

## Global Commands

You can run these commands from the root of the repository. Turborepo will execute them in the relevant workspaces.

-   `pnpm build`: Builds all apps and packages.
-   `pnpm lint`: Lints all apps and packages.
-   `pnpm dev`: Runs all applications in development mode. (Note: This might be resource-intensive. It's often better to run a specific app's dev server).
-   `pnpm check-types`: Runs TypeScript type checking across all relevant packages.
-   `pnpm format`: Formats all supported files with Prettier.

## Workspace Details

### Frontend Apps (`apps/web`, `apps/docs`)

The `apps/web` and `apps/docs` directories contain Next.js applications written in TypeScript.

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

The `apps/mobile` directory contains a React Native application built with **Expo**.

**To run the mobile app:**
First, ensure you have the Expo CLI installed (`pnpm install -g expo-cli`) and meet the requirements for React Native development (Android Studio, Xcode, etc.).

```bash
# Navigate to the mobile app directory
cd apps/mobile

# Start the development server
pnpm start

# Then, from the Expo Dev Tools in your browser, you can:
# - Run on Android device/emulator
# - Run on iOS simulator
# - Run in web browser
```

Alternatively, you can use the direct commands from `apps/mobile`:
- `pnpm android`
- `pnpm ios`

**Build:**
The app is built for production using Expo Application Services (EAS). These commands should be run from `apps/mobile`:
- `pnpm build:android`
- `pnpm build:ios`

### Python API (`apps/api`)

The `apps/api` directory contains a FastAPI backend.

**Setup:**
1.  Navigate to the directory: `cd apps/api`
2.  Create a virtual environment: `python -m venv .venv`
3.  Activate it: `source .venv/bin/activate`
4.  Install dependencies: `pip install -r requirements.txt`
5.  Create a `.env` file for environment variables. This app requires `GEMINI_API_KEY` and `FAL_API_KEY`. These are secrets and will not be available in the repository.

**Running the server:**
From within `apps/api` with the virtual environment activated:

```bash
uvicorn main:app --reload --port 8000
```

**Testing:**
Run tests using `pytest`:

```bash
# from apps/api directory
pytest
```

**Linting and Formatting:**
This project uses `black`, `isort`, `flake8`, and `mypy`.

```bash
# from apps/api directory
black .
isort .
flake8 .
mypy .
```

### Rust Projects

The repository contains two main Rust projects: `apps/agents` and `apps/contracts`. You will need the Rust toolchain installed (`rustup`).

#### AI Agents (`apps/agents`)

This project contains multiple agent binaries.

**Commands (run from `apps/agents`):**
-   `cargo check`: Check the code for errors.
-   `cargo build`: Build all agent binaries.
-   `cargo build --bin governance-agent`: Build a specific binary.
-   `cargo run --bin governance-agent`: Run a specific binary.
-   `cargo test`: Run tests.
-   `cargo clippy`: Lint the code.

#### Smart Contracts (`apps/contracts`)

This is a workspace containing NEAR smart contracts. The contracts are in `apps/contracts/voting`, `apps/contracts/staking`, etc.

**Commands (run from a specific contract directory, e.g., `apps/contracts/voting`):**
-   `cargo check`: Check the code.
-   `cargo test`: Run unit tests. Note that these tests run in a mocked environment and do not require a connection to a live NEAR network.
-   `cargo clippy`: Lint the code.

**Building Contracts:**
NEAR contracts must be compiled to Wasm.

```bash
# From a contract directory like apps/contracts/voting
cargo build --target wasm32-unknown-unknown --release
```

### Shared Packages

The `packages/` directory contains shared code for the TypeScript applications.

-   **`@repo/ui`**: A shared React component library built with Radix UI and `tsup`.
-   **`@groupweave/common-types`**: Shared TypeScript types.
-   **`@repo/eslint-config`**, **`@repo/tailwind-config`**, **`@repo/typescript-config`**: Shared configurations for linting, styling, and TypeScript.

These packages are automatically built as dependencies when you run `pnpm build` from the root.

## Dependencies

To install all dependencies for the monorepo, run this command from the root directory:

```bash
pnpm install
```

This will install dependencies for all workspaces.

## Coding Style & Linting

-   **TypeScript/JavaScript**: Run `pnpm format` to format the code using Prettier. Run `pnpm lint` to check for code quality issues.
-   **Python**: Use `black` and `isort` for formatting, and `flake8` and `mypy` for linting.
-   **Rust**: Use `rustfmt` (usually via `cargo fmt`) for formatting and `cargo clippy` for linting.
