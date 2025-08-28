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

### Frontend Apps (`apps/creation`, `apps/participation`, `apps/dashboard`, `apps/docs`)

The frontend applications are Next.js applications written in TypeScript.

-   **`apps/creation`**: The application for creating new voting rounds.
-   **`apps/participation`**: The application for participating in voting rounds.
-   **`apps/dashboard`**: The application for viewing the results of voting rounds.
-   **`apps/docs`**: The documentation website.

**To run a specific frontend app:**

```bash
# To run the creation app (available at http://localhost:3003)
pnpm --filter creation dev

# To run the participation app (available at http://localhost:3002)
pnpm --filter participation dev

# To run the dashboard app (available at http://localhost:3004)
pnpm --filter dashboard dev

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

## Coding Style & Linting
### Javascript / Typescript
- Formatted with Prettier, linted with ESLint. Follows a specific import order.

### Python
- Formatted with black and isort, linted with flake8 and mypy.

### Rust
- Formatted with rustfmt, linted with cargo clippy.
