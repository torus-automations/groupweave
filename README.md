# GroupWeave

GroupWeave is the open-source infrastructure for user-owned AI, designed to enable co-creation and co-immersion in generative AI content. Currently developing on NEAR, with plans to integrate Shade agents as customizable, semi-autonomous assistants for multimodal content understanding, moderation, and curation.

## Getting Started

To get started with GroupWeave development, you'll need to have Node.js, pnpm, Rust, and Python installed. Once you have the prerequisites, follow the steps below.

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/torus-automations/groupweave.git
    cd groupweave
    ```

2.  **Install dependencies:**
    ```bash
    pnpm install
    ```

3.  **Build all packages:**
    ```bash
    pnpm build
    ```

4.  **Run an application:**
    To run a specific application, use the `pnpm --filter <app-name> dev` command. For example, to run the `creation` app:
    ```bash
    pnpm --filter creation dev
    ```

For more detailed instructions on the development environment, commands, and project structure, please refer to our comprehensive developer and agent guide:

**[📜 AGENTS.md](./AGENTS.md)**

## Project Structure

This project is a monorepo managed with Turborepo and pnpm workspaces.

-   `apps/`: Contains the applications, including Next.js frontends, a React Native mobile app, a Python API, and Rust-based agents.
-   `packages/`: Contains shared packages used by the applications, such as UI components, common types, and configs.

A detailed breakdown of all workspaces can be found in the **[AGENTS.md](./AGENTS.md)** file.

## Developer Contributions

We welcome contributions! Please fork the repository, create a branch, and make your changes. Once your feature or fix is ready, please make a pull request for review.

For details on how to work with the shared UI component library, please see our **[UI Component Guide](./UI_COMPONENTS.md)**.

## Automated Documentation

This repository uses a script to generate key documentation files (`AGENTS.md`, `CLAUDE.md`, etc.) from a single source of truth: `docs.json`.

If you make changes that require documentation updates (e.g., adding a new workspace), please:
1.  Update the `docs.json` file.
2.  Run the generation script: `python generate_docs.py`.
3.  Commit the changes to `docs.json` and the generated files.

## Roadmap

The following features are currently being developed:

*   Betting as a mechanism for content curation/moderation with reward and punishment.
*   User-owned AI and personalization with confidential AI models and trusted execution environments.
*   Confidential voting/betting and tally counting with Zero Knowledge Proofs (ZKP).

## Connect with us
[![Website](https://readmecodegen.vercel.app/api/social-icon?name=Link)](https://www.torus-automations.xyz/)
[![Discord](https://readmecodegen.vercel.app/api/social-icon?name=Discord)](https://discord.gg/wgN9HhUM)
[![Telegram](https://readmecodegen.vercel.app/api/social-icon?name=Telegram)](https://t.me/torusautomations)
[![LinkedIn](https://readmecodegen.vercel.app/api/social-icon?name=LinkedIn)](https://www.linkedin.com/in/company/torus-automations/)
