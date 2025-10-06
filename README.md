# GroupWeave

GroupWeave is the open infrastructure layer for user-owned AI experiences. The project empowers collaborative content creation and co-immersion across modalities, with NEAR as the first-chain deployment target and Shade agents extending into trusted execution environments.

## Current Status (October 6, 2025)

- Active app and smart contract development now resides in the private **DreamWeave** repository while we complete a comprehensive security audit.
- The public `apps/*` workspaces remain as stubs; build, deploy, and contract commands are intentionally disabled until the audited code is re-open-sourced.
- The `apps/creation`, `apps/dashboard`, and `apps/participation` Next.js projects have been merged into a single DreamWeave web experience. Updated sources will return to this repository after the audit deliverables are incorporated.
- NEAR contracts previously tracked under `apps/contracts` were relocated for audit hardening. A sanitized reference implementation will be reintroduced here alongside audit notes.
- Shared packages (`packages/ui`, `packages/common-types`, etc.) and infrastructure tooling continue to evolve in this monorepo, including documentation automation and agent-facing guidance.

## Working in This Repository

While the application code is private, contributors can still collaborate on:

- Documentation updates (`docs.json`, `AGENTS.md`, `CLAUDE.md`) and developer onboarding flows.
- Shared TypeScript packages and linting/formatting presets.
- DevOps scaffolding (Turborepo tasks, pnpm scripts, Docker tooling) that does not expose private implementation details.

If you need to coordinate with the DreamWeave effort or request preview access during the audit window, please open an issue or reach out via the channels below.

## Contribution Process

1. Fork the repository and create a feature branch for documentation, tooling, or shared package improvements.
2. Confirm that your changes do not rely on private DreamWeave assets.
3. Ensure `pnpm lint`, `pnpm check-types`, and relevant tests pass for the packages you touched.
4. Submit a pull request with conventional commit formatting. Maintainers will review for alignment with the audit timeline.

## Connect with us
[![Website](https://readmecodegen.vercel.app/api/social-icon?name=Link)](https://www.torus-automations.xyz/)
[![Discord](https://readmecodegen.vercel.app/api/social-icon?name=Discord)](https://discord.gg/wgN9HhUM)
[![Telegram](https://readmecodegen.vercel.app/api/social-icon?name=Telegram)](https://t.me/torusautomations)
[![LinkedIn](https://readmecodegen.vercel.app/api/social-icon?name=LinkedIn)](https://www.linkedin.com/in/company/torus-automations/)

## Automated Documentation

To ensure consistency across the various documentation files (`AGENTS.md`, `CLAUDE.md`, etc.), this repository uses a script to generate them from a single source of truth (`docs.json`).

When you make changes to the repository that require documentation updates (e.g., adding a new workspace, changing build commands), you should:
1.  Update the `docs.json` file with the new information.
2.  Run the generation script to update the documentation files:
    ```bash
    python generate_docs.py
    ```
3.  Commit the changes to `docs.json` and the generated documentation files.

## Roadmap

The following priorities are in flight as part of the DreamWeave audit cycle:

- Hardened launch of the merged DreamWeave web experience with modular content pipelines.
- Security audit remediation for NEAR staking, voting, and ZKP verification contracts.
- Expanded Shade agent integrations for confidential, user-owned inference paths.

Once the audit closes, the corresponding applications and contracts will be re-synced here together with migration notes.
