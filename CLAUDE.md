# GroupWeave - Claude Code Instructions

## Repository Context
- This is an open-source platform for user-owned AI, focusing on co-creation and co-immersion in generative content.
- Complex monorepo with web, mobile, backend API, smart contracts, and AI agents.
- Follows strict TypeScript practices with a zero-warnings policy.
- Uses a shared component library (`@repo/ui`) built with Radix UI.

## Monorepo Tooling
- **pnpm**: Package manager - replace all `npm`/`yarn` commands with `pnpm`
- **Turborepo**: Manages tasks and build pipelines across the monorepo

## Development Preferences
- Always run `pnpm lint` and `pnpm check-types` before committing.
- Follow existing component patterns in `@repo/ui`.
- Test mobile changes across platforms when possible.
- Coordinate changes across workspace dependencies.
- Use `pnpm --filter <workspace>` for workspace-specific commands.

## Global Commands (run from root)
- `pnpm install` - Install all dependencies
- `pnpm build` - Build all apps and packages
- `pnpm lint` - Lint all apps and packages
- `pnpm check-types` - Run TypeScript type checking
- `pnpm format` - Format with Prettier
- `pnpm dev` - Run all apps (resource-intensive, prefer specific apps)

## Workspace Structure

### Frontend Apps
- **`apps/web`**: Main GroupWeave web app (Next.js/TypeScript)
  - Run: `pnpm --filter web dev` (http://localhost:3002)
- **`apps/docs`**: Documentation website (Next.js/TypeScript)  
  - Run: `pnpm --filter docs dev` (http://localhost:3001)

### Mobile App (`apps/mobile`)
- React Native with Expo
- Commands from `apps/mobile`: `pnpm start`, `pnpm android`, `pnpm ios`
- Build: `pnpm build:android`, `pnpm build:ios`

### Python API (`apps/api`)
- FastAPI backend
- Setup: `cd apps/api && python -m venv .venv && source .venv/bin/activate && pip install -r requirements.txt`
- Run: `uvicorn main:app --reload --port 8000`
- Test: `pytest`
- Lint: `black . && isort . && flake8 . && mypy .`
- Requires: `GEMINI_API_KEY` and `FAL_API_KEY` env vars

### Rust Projects
- **`apps/agents`**: AI agent binaries
  - Commands: `cargo check`, `cargo build`, `cargo test`, `cargo clippy`
- **`apps/contracts`**: NEAR smart contracts
  - Commands from contract dirs: `cargo check`, `cargo test`, `cargo clippy`  
  - Build WASM: `cargo build --target wasm32-unknown-unknown --release`

### Shared Packages (`packages/`)
- **`@repo/ui`**: React component library (Radix UI + tsup)
- **`@groupweave/common-types`**: Shared TypeScript types
- **`@repo/eslint-config`**, **`@repo/tailwind-config`**, **`@repo/typescript-config`**: Shared configs

## Key Files
- `/turbo.json` - Build pipeline configuration
- `/apps/web/components/` - Main UI components
- `/packages/ui/` - Shared component library
- `/apps/api/requirements.txt` - Python dependencies
- `/apps/*/Cargo.toml` - Rust project configs

## Coding Standards
- **TypeScript/JavaScript**: Prettier formatting, ESLint rules
- **Python**: black, isort, flake8, mypy
- **Rust**: rustfmt, cargo clippy
