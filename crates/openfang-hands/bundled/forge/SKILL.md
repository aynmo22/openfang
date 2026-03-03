---
domain: developer-tools
version: "1.0"
sources:
  - "OpenFang Hands Documentation"
  - "FastMCP Documentation"
  - "OpenRouter Model Catalog"
---

# Forge — Scaffolding Domain Knowledge

## OpenFang Hand Structure

A bundled Hand directory contains two files:

| File | Required | Purpose |
|------|----------|---------|
| HAND.toml | Yes | Manifest — identity, tools, settings, agent config with inline system_prompt, dashboard metrics |
| SKILL.md | Yes | Domain knowledge — expert reference material injected into context |

### HAND.toml Top-Level Fields

| Field | Type | Purpose |
|-------|------|---------|
| `id` | string | Unique identifier (lowercase-hyphenated) |
| `name` | string | Human-readable name |
| `description` | string | What this Hand does |
| `category` | enum | productivity, data, content, communication, security, development |
| `icon` | string | Emoji or icon identifier |
| `tools` | array | List of tool names the agent can use |

### Settings Array

Each `[[settings]]` entry creates a configurable option in the activation wizard:

```toml
[[settings]]
key = "setting_key"           # Unique identifier
label = "Display Label"       # Shown in UI
description = "Help text"     # Tooltip
setting_type = "select"       # select | text | toggle
default = "default_value"     # Default value

[[settings.options]]          # Only for select type
value = "option_value"
label = "Option Label"
```

### Agent Configuration

```toml
[agent]
name = "hand-name"
description = "Brief description"
module = "builtin:chat"
provider = "default"          # Inherits user's global config
model = "default"             # Inherits user's global config
max_tokens = 16384
temperature = 0.3             # 0.1-0.3 for precision, 0.5-0.7 for creative
max_iterations = 50           # Max LLM calls per run
system_prompt = """..."""     # Inline multi-line string
```

### Dashboard Metrics

```toml
[dashboard]
[[dashboard.metrics]]
label = "Display Name"
memory_key = "hand_id_metric_name"  # Key in agent's memory
format = "number"                    # number | duration | bytes
```

## Available OpenFang Tools

**Filesystem:** file_read, file_write, file_list, file_delete, directory_create

**Execution:** shell_exec, python_exec

**Network:** web_fetch, web_search, api_call

**Communication:** agent_send, agent_list, channel_send

**Memory:** memory_store, memory_recall, memory_search, memory_delete

**Knowledge Graph:** knowledge_add_entity, knowledge_add_relation, knowledge_query

**Scheduling:** schedule_create, schedule_list, schedule_delete

**Browser:** browser_navigate, browser_click, browser_type, browser_screenshot, browser_read_page, browser_close

## MCP Server Pattern (FastMCP)

Standard structure:
```
{name}-mcp/
├── server.py          # FastMCP server, tool registration
├── tools.py           # Tool implementations
├── config.json        # Connection config, env var references
└── requirements.txt   # Pinned dependencies
```

**server.py template:**
```python
from fastmcp import FastMCP
mcp = FastMCP("{name}")
from tools import *
if __name__ == "__main__":
    mcp.run()
```

**tools.py template:**
```python
from fastmcp import tool

@tool
def tool_name(param: str, limit: int = 10) -> dict:
    """Describe what the tool does.

    Args:
        param: Description of parameter
        limit: Maximum results to return
    """
    pass
```

## Python Pipeline Pattern

Standard structure:
```
{name}-pipeline/
├── main.py            # argparse entry point, logging
├── config.yaml        # Externalized configuration
└── requirements.txt   # Pinned dependencies
```

## System Prompt Design — The Five Rules

1. **Be procedural, not descriptive.** Don't say "you are an invoice processor." Say "Phase 1: Check the input directory for new files."

2. **Define phases.** Break work into numbered phases with concrete steps.

3. **Include decision trees.** "If X exceeds threshold, do Y. If below, do Z."

4. **Specify error recovery.** Every phase needs a fallback.

5. **Set quality gates.** "Do not proceed to Phase 3 until condition is met."

## OpenRouter Model Reference

| Model ID | Strengths | Best For |
|----------|-----------|----------|
| `deepseek/deepseek-chat` | Fast, precise, strong coder | Code generation, TOML, configs |
| `minimax/minimax-01` | Long context, reasoning | Complex multi-step tasks |
| `thudm/glm-4` | Balanced, multilingual | General purpose |
| `moonshot/moonshot-v1-128k` | 128K context | Long-form writing |

## Known Pitfalls

1. **Model IDs must be exact.** Use `deepseek/deepseek-chat` not `deepseek-chat`.

2. **System prompts under 500 words** produce unreliable autonomous behavior.

3. **Memory key collisions** — always prefix with the Hand id (e.g., `forge_scaffolds_generated`).

4. **max_iterations caps LLM calls** — set conservatively to avoid runaway costs.

5. **Settings with options array** create dropdown selectors in the dashboard UI.

6. **TOML escape sequences** — use `\U0001F528` for emoji in icon field.

7. **Bundled Hands compile into binary** via `include_str!()`. Runtime Hands load from `~/.openfang/hands/`.
