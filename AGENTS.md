# AGENTS.md

This file provides instructions for AI agents working on the GroupWeave monorepo.

## Global Commands

-   **Install all dependencies:**
    ```bash
    pnpm install
    ```
-   **Build all packages:**
    ```bash
    pnpm build
    ```
-   **Run all linters:**
    ```bash
    pnpm lint
    ```
-   **Check all types:**
    ```bash
    pnpm check-types
    ```

---

## Project Structure

This is a monorepo using Turborepo.

-   `apps/`: Contains the applications, such as the Next.js frontend apps, the mobile app, and the Python API.
-   `packages/`: Contains shared packages used by the applications, such as UI components, common types, and configs.

---

## Workspace Instructions

### Frontend Apps (`apps/creation`, `apps/participation`, `apps/dashboard`, `apps/docs`)

These are Next.js applications.

-   **Run a specific app (e.g., `creation`):**
    ```bash
    pnpm --filter creation dev
    ```
-   **Run tests for a specific app:**
    ```bash
    pnpm --filter creation test
    ```

### Mobile App (`apps/mobile`)

This is a React Native app using Expo.

-   **Run the app:**
    ```bash
    # From apps/mobile directory
    pnpm start
    ```
-   **Run on iOS:**
    ```bash
    # From apps/mobile directory
    pnpm ios
    ```
-   **Run on Android:**
    ```bash
    # From apps/mobile directory
    pnpm android
    ```

### Python API (`apps/api`)

This is a FastAPI application.

-   **Setup and run:**
    ```bash
    cd apps/api
    python -m venv .venv
    source .venv/bin/activate
    pip install -r requirements.txt
    uvicorn main:app --reload --port 8000
    ```
-   **Run tests:**
    ```bash
    # From apps/api directory
    pytest
    ```
-   **Run linters:**
    ```bash
    # From apps/api directory
    black .
    isort .
    flake8 .
    mypy .
    ```

### Rust Projects (`apps/agents`, `apps/contracts`)

-   **Check code:** `cargo check`
-   **Build:** `cargo build`
-   **Run tests:** `cargo test`
-   **Run linter:** `cargo clippy`

---

## Code Style

-   **TypeScript/JavaScript:** Prettier and ESLint. Import order is enforced.
-   **Python:** `black`, `isort`, `flake8`, `mypy`.
-   **Rust:** `rustfmt`, `clippy`.

---
