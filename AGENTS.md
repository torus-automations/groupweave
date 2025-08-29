# AGENTS.md

This file provides instructions for AI agents working on the GroupWeave monorepo.

## Project Structure

This is a monorepo using Turborepo.

-   `apps/`: Contains the applications, such as the Next.js frontend apps, the mobile app, and the Python API.
-   `packages/`: Contains shared packages used by the applications, such as UI components, common types, and configs.

---

## Setup and Global Commands

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

## Testing Instructions

-   **Run tests for a specific frontend app (e.g., `creation`):**
    ```bash
    pnpm --filter creation test
    ```
-   **Run tests for the Python API:**
    ```bash
    # From apps/api directory
    pytest
    ```
-   **Run tests for Rust projects:**
    ```bash
    # From the specific project directory (e.g., apps/agents)
    cargo test
    ```
-   Always run tests before submitting a change.

---

## Workspace-Specific Commands

### Frontend Apps (`apps/*`)

-   **Run a specific app (e.g., `creation`):**
    ```bash
    pnpm --filter creation dev
    ```

### Mobile App (`apps/mobile`)

-   **Run the app:** `cd apps/mobile && pnpm start`
-   **Run on iOS:** `cd apps/mobile && pnpm ios`
-   **Run on Android:** `cd apps/mobile && pnpm android`

### Python API (`apps/api`)

-   **Setup and run:**
    ```bash
    cd apps/api
    python -m venv .venv
    source .venv/bin/activate
    pip install -r requirements.txt
    uvicorn main:app --reload --port 8000
    ```

### Rust Projects (`apps/agents`, `apps/contracts`)

-   **Check code:** `cargo check`
-   **Build:** `cargo build`
-   **Run linter:** `cargo clippy`

---

## PR Instructions

-   **Title format:** `type(scope): description` (e.g., `feat(creation): add new button`)
-   Run `pnpm lint` and `pnpm check-types` before submitting.
-   Ensure all tests pass.

---

## Code Style

-   **TypeScript/JavaScript:** Prettier and ESLint. Import order is enforced.
-   **Python:** `black`, `isort`, `flake8`, `mypy`.
-   **Rust:** `rustfmt`, `clippy`.
