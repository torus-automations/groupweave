# GroupWeave

AI content generation is advancing at an incredible pace, leading to a significant shift in content creation and consumption.

GroupWeave is the open-source infrastructure for user-owned AI, designed to enable co-creation and co-immersion in generative AI content. Currently developing on NEAR, with plans to integrate Shade agents as customizable, semi-autonomous assistants for multimodal content understanding, moderation, and curation.

## Developer Contributions

Please fork the repository, then create a branch and make the changes you wish to propose. Once the feature or fix is successfully implemented, resolve any conflicts with main locally. Finally, make a pull request and submit for review. Code contributions have to be approved by the maintainer(s), one at this time, and two when the project has grown. 

## How to use

Do not deploy any of the apps or smart contracts yet. The standards for merging will increase in a week. Please check back later in August.

This big project is developed as a monorepo with proper tooling, centralized documentation, and strict coding guidelines. Notes will be provided on conventions, AGENTS/CLAUDE.md, Docker, and Turborepo. AI agents should have a high efficacy with this setup.

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

The following features are currently being developed:

*   Betting as a mechanism for content curation/moderation with reward and punishment 
*   User-owned AI and personalization with confidential AI models and trusted execution environments
*   To be decided: Confidential voting/betting and tally counting with Zero Knowledge Proofs (ZKP)
