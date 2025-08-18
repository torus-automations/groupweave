import json

def generate_agents_md(data):
    """
    Generate the markdown content for the AGENTS.md file.

    Args:
        data (dict): A dictionary containing project information. Expected keys include:
            - 'projectName' (str): The name of the project.
            - 'projectDescription' (str): A description of the project.
            - 'tooling' (list of dict): List of tools, each with 'name' and 'description'.
            - 'globalCommands' (list of dict): List of commands, each with 'command' and 'description'.
            - 'workspaces' (list of dict): List of workspaces, each with 'name' and 'description'.
            - 'codingStyle' (dict): Mapping of language names to style descriptions.

    Returns:
        str: The formatted markdown content for the AGENTS.md file.
    """
    content = f"# {data['projectName']} Monorepo - Agent & Developer Instructions\n\n"
    content += f"{data.get('agentsIntroduction', 'This document provides comprehensive instructions for developers and AI agents working on this monorepo.')}\n\n"
    content += "## Overview\n"
    content += f"{data['projectDescription']}\n\n"
    content += "## Monorepo Tooling\n"
    for tool in data['tooling']:
        content += f"### {tool['name']}\n{tool['description']}\n\n"
    content += "## Global Commands\n"
    for cmd in data['globalCommands']:
        content += f"- `{cmd['command']}`: {cmd['description']}\n"
    content += "\n"
    content += "## Workspace Details\n"
    for ws in data['workspaces']:
        content += f"- **`{ws['name']}`**: {ws['description']}\n"
    content += "\n"
    content += "## Coding Style & Linting\n"
    for lang, style in data['codingStyle'].items():
        content += f"### {lang.replace('/', ' / ').title()}\n- {style}\n\n"
    return content

def generate_claude_md(data):
    """Generates the content for CLAUDE.md."""
    content = f"# Claude Instructions for {data['projectName']}\n\n"
    content += "For all instructions on working with this monorepo, please refer to the main developer and agent guide:\n\n"
    content += "**[./AGENTS.md](./AGENTS.md)**\n\n"
    content += "This file serves as the single source of truth for repository setup, development workflows, and coding standards."
    return content

def generate_copilot_instructions(data):
    """Generates the content for .github/copilot-instructions.md."""
    content = f"# GitHub Copilot Instructions for {data['projectName']}\n\n"
    content += "For all instructions on working with this monorepo, please refer to the main developer and agent guide:\n\n"
    content += "**[../AGENTS.md](../AGENTS.md)**\n\n"
    content += "This file serves as the single source of truth for repository setup, development workflows, and coding standards."
    return content

def main():
    """Main function to generate all documentation files."""
    with open('docs.json', 'r') as f:
    try:
        with open('docs.json', 'r') as f:
            data = json.load(f)
    except FileNotFoundError:
        print("Error: 'docs.json' file not found. Please ensure the file exists in the current directory.")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error: Failed to parse 'docs.json': {e}")
        sys.exit(1)
    except Exception as e:
        print(f"An unexpected error occurred while reading 'docs.json': {e}")
        sys.exit(1)

    with open('AGENTS.md', 'w') as f:
        f.write(generate_agents_md(data))

    with open('CLAUDE.md', 'w') as f:
        f.write(generate_claude_md(data))

    with open('.github/copilot-instructions.md', 'w') as f:
        f.write(generate_copilot_instructions(data))

    print("Documentation files generated successfully.")

if __name__ == "__main__":
    main()
