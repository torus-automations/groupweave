# GroupWeave Monorepo - Copilot Instructions

This document provides instructions for GitHub Copilot to ensure consistency and correctness when working within the GroupWeave monorepo.

## High-Level Details

- **Repository Purpose**: GroupWeave is an open-source platform for user-owned AI, focusing on co-creation and co-immersion in generative content.
- **Project Type**: This is a monorepo containing multiple applications and services.
- **Languages**: TypeScript, Python, Rust
- **Frameworks**: Next.js, React Native (with Expo), FastAPI
- **Tooling**: pnpm, Turborepo

## Build Instructions

- **Installation**: Run `pnpm install` from the root of the repository to install all dependencies for all workspaces.
- **Build**: Run `pnpm build` to build all apps and packages.
- **Linting**: Run `pnpm lint` to lint all apps and packages.
- **Formatting**: Run `pnpm format` to format all supported files with Prettier.
- **Type Checking**: Run `pnpm check-types` to run TypeScript type checking across all relevant packages.

### Workspace-Specific Commands

- **Frontend Apps (`apps/web`, `apps/docs`)**:
    - To run the web app: `pnpm --filter web dev` (available at http://localhost:3002)
    - To run the docs site: `pnpm --filter docs dev` (available at http://localhost:3001)
- **Mobile App (`apps/mobile`)**:
    - Navigate to `apps/mobile` and run `pnpm start`.
- **Python API (`apps/api`)**:
    - Navigate to `apps/api`, create a virtual environment, activate it, and run `pip install -r requirements.txt`.
    - Run the server with `uvicorn main:app --reload --port 8000`.
- **Rust Projects (`apps/agents`, `apps/contracts`)**:
    - Use standard `cargo` commands (`check`, `build`, `run`, `test`, `clippy`) from within the project directories.

## Project Layout

- `apps/`: Contains the individual applications.
    - `web/`: The main Next.js web application.
    - `docs/`: The documentation website.
    - `mobile/`: The React Native mobile app.
    - `api/`: The Python FastAPI backend.
    - `agents/`: Rust-based AI agents.
    - `contracts/`: NEAR smart contracts written in Rust.
- `packages/`: Contains shared code and configurations.
    - `ui/`: Shared React component library.
    - `common-types/`: Shared TypeScript types.
    - `eslint-config/`: Shared ESLint configuration.
    - `tailwind-config/`: Shared Tailwind CSS configuration.
    - `typescript-config/`: Shared TypeScript configuration.

## Coding Standards

### JavaScript/TypeScript

- **Import Style**:
    - Always use `import type` when importing only types.
    - **Good**: `import type { SomeType } from './types';`
    - **Bad**: `import { SomeType } from './types';`
- **Import Order**: While not strictly enforced by the linter, please follow this general order:
    1. React imports
    2. External library imports
    3. Internal absolute path imports (e.g., from `@repo/ui`)
    4. Relative path imports
- **General**: Follow strict TypeScript practices with a zero-warnings policy. Run `pnpm lint` and `pnpm check-types` before committing.

### Python

- Use `black` and `isort` for formatting.
- Use `flake8` and `mypy` for linting.

### Rust

- Use `cargo fmt` for formatting.
- Use `cargo clippy` for linting.
