---
domain: developer-tools
version: "1.0"
sources:
  - "OpenFang Hands Documentation"
  - "OpenFang CONTRIBUTING.md"
  - "OpenRouter Model Catalog"
  - "FastMCP Documentation"
---

# Forge — Scaffolding Domain Knowledge

## OpenFang Hand Structure

A Hand directory contains three files:

| File | Required | Purpose |
|------|----------|---------|
| HAND.toml | Yes | Manifest — identity, tools, settings, schedule, metrics |
| system-prompt.md | Yes | Operational playbook — multi-phase procedures |
| SKILL.md | Recommended | Domain knowledge — expert reference for context injection |

### HAND.toml Required Sections

| Section | Purpose |
|---------|---------|
| `[hand]` | id, name, description, category, icon, version |
| `[hand.requires]` | tools list |
| `[hand.settings]` | User-configurable options (type, default, options, min/max, description) |
| `[hand.agent]` | model, temperature, max_iterations |
| `[hand.schedule]` | cron expression |
| `[hand.dashboard]` | metrics array |
| `[hand.triggers]` | Event-based activation (optional) |
| `[hand.chain]` | Multi-Hand orchestration (optional) |

### Setting Types

Supported: `string`, `bool`, `int`, `float`, `string[]`

Each setting can have: `default`, `options` (enum), `min`/`max` (numeric), `description`

### Available OpenFang Tools

Filesystem: file_read, file_write, file_list, file_delete, directory_create
Execution: shell_exec, python_exec
Network: web_fetch, web_search, api_call
Communication: agent_send, agent_list, channel_send
Memory: memory_read, memory_write, memory_search, memory_delete
Media: image_analyze, tts_speak, audio_transcribe
Browser: browser_navigate, browser_click, browser_screenshot
Scheduling: schedule_create, schedule_list, schedule_delete

## OpenRouter Model Reference

Al's approved models for workflow LLM calls (never Claude API):

| Model ID | Strengths | Best For |
|----------|-----------|----------|
| `deepseek/deepseek-chat` | Fast, precise, strong coder | Code generation, TOML, configs, data processing |
| `minimax/minimax-01` | Long context, reasoning | Complex multi-step tasks, analysis |
| `thudm/glm-4` | Balanced, multilingual | General purpose, moderate complexity |
| `moonshot/moonshot-v1-128k` | 128K context, long-form writing | System prompts, SKILL.md, research reports |

## MCP Server Pattern (Python/FastMCP)

Standard directory structure:
```
{name}-mcp/
├── server.py          # FastMCP server, tool registration
├── tools.py           # Tool implementations
├── config.json        # Connection config, env var references
└── requirements.txt   # Pinned dependencies
```

Server template:
```python
from fastmcp import FastMCP
mcp = FastMCP("{name}")
from tools import *
if __name__ == "__main__":
    mcp.run()
```

Tool definition template:
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

Standard directory structure:
```
{name}-pipeline/
├── main.py            # argparse entry point, logging, config loader
├── config.yaml        # Externalized configuration
└── requirements.txt   # Pinned dependencies
```

Entry point template:
```python
#!/usr/bin/env python3
import argparse, logging, yaml, sys
from pathlib import Path

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

def load_config(path="config.yaml"):
    with open(path) as f:
        return yaml.safe_load(f)

def main(args):
    config = load_config(args.config)
    try:
        # Pipeline logic
        pass
    except Exception as e:
        logger.error(f"Failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", default="config.yaml")
    main(parser.parse_args())
```

## System Prompt Design Principles

From OpenFang docs — the five rules:

1. **Be procedural, not descriptive.** Don't say "you are an invoice processor." Say "Phase 1: Check the input directory for new files. For each file..."
2. **Define phases.** Break work into numbered phases with concrete steps. Sequential unless a condition routes elsewhere.
3. **Include decision trees.** "If X exceeds threshold, do Y. If below threshold, do Z."
4. **Specify error recovery.** Every phase needs a fallback. Agents without recovery instructions loop or hallucinate.
5. **Set quality gates.** "Do not proceed to Phase 3 until condition is met."

## Al's Environment Context

- Infrastructure: EC2, Elasticsearch (~2M creator records)
- Orchestration: N8N with OpenRouter models
- Active projects: Creator Search MCP, RFI Agent, sentiment analysis, UGC pipeline, brand safety
- Deployment: Local dev → EC2 (same VPC as Elasticsearch)
- Communication: Slack primary
- Team: 7-person tech team, contractor-augmented
- Company: Open Influence, $75M creator marketing, Fortune 1000 clients

## Known Pitfalls

1. OpenRouter model IDs must match the catalog exactly. `deepseek/deepseek-chat` not `deepseek-chat`.
2. TOML boolean shorthand (`require_approval = false`) works in v0.1.3+.
3. System prompts under 500 words produce unreliable autonomous behavior.
4. Memory namespace collisions — always prefix keys with the Hand id.
5. Settings with `options` array create dropdown selectors in the dashboard.
6. The `max_iterations` in `[hand.agent]` caps how many LLM calls per run — set conservatively.
7. Hands are compiled into the binary via `include_str!()` — for custom Hands installed at runtime, they load from disk at `~/.openfang/hands/`.
