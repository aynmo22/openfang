# Forge Hand Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Forge as a bundled Hand that scaffolds OpenFang Hands, MCP servers, and Python pipelines from natural language descriptions.

**Architecture:** Forge is added as the 8th bundled Hand in `crates/openfang-hands/bundled/forge/`. It registers in `bundled.rs` and compiles into the binary. Users activate it via the dashboard, chat with it to describe what they need, and it stages files for review before deployment.

**Tech Stack:** Rust (OpenFang codebase), TOML (Hand manifest), Markdown (system prompt + skill content)

**Reference Design:** `docs/plans/2026-03-02-forge-hand-design.md`

---

## Task 1: Git Setup — Add Upstream Remote

**Files:**
- None (git configuration only)

**Step 1: Check if upstream remote exists**

```bash
git remote -v
```

Expected: May or may not show `upstream`. If it does, skip to Task 2.

**Step 2: Add upstream remote**

```bash
git remote add upstream https://github.com/RightNow-AI/openfang.git
```

**Step 3: Fetch upstream**

```bash
git fetch upstream
```

Expected: Fetches branches from upstream repository.

**Step 4: Verify remote was added**

```bash
git remote -v
```

Expected: Shows both `origin` (openfang-oi) and `upstream` (RightNow-AI/openfang).

---

## Task 2: Git Setup — Create Feature Branch

**Files:**
- None (git configuration only)

**Step 1: Ensure on main branch**

```bash
git checkout main
```

**Step 2: Create feature branch**

```bash
git checkout -b feature/forge-hand
```

**Step 3: Verify branch**

```bash
git branch --show-current
```

Expected: `feature/forge-hand`

---

## Task 3: Create Forge Directory Structure

**Files:**
- Create: `crates/openfang-hands/bundled/forge/` (directory)

**Step 1: Create the forge directory**

```bash
mkdir -p crates/openfang-hands/bundled/forge
```

**Step 2: Verify directory exists**

```bash
ls -la crates/openfang-hands/bundled/
```

Expected: Shows `forge` directory alongside `browser`, `clip`, `collector`, etc.

---

## Task 4: Create HAND.toml

**Files:**
- Create: `crates/openfang-hands/bundled/forge/HAND.toml`
- Reference: `crates/openfang-hands/bundled/browser/HAND.toml` (for format)

**Step 1: Write HAND.toml**

Create `crates/openfang-hands/bundled/forge/HAND.toml` with:

```toml
id = "forge"
name = "Forge"
description = "Personal builder's scaffolding agent — generates OpenFang Hands, MCP servers, and Python pipelines from natural language descriptions"
category = "development"
icon = "\U0001F528"
tools = [
    "file_read", "file_write", "file_list", "file_delete",
    "shell_exec", "memory_store", "memory_recall"
]

# ─── Configurable settings ───────────────────────────────────────────────────

[[settings]]
key = "generation_model"
label = "Generation Model"
description = "Which model to use for scaffold generation"
setting_type = "select"
default = "deepseek/deepseek-chat"

[[settings.options]]
value = "deepseek/deepseek-chat"
label = "DeepSeek V3.2 (fast, precise)"

[[settings.options]]
value = "minimax/minimax-01"
label = "MiniMax M2.5 (reasoning)"

[[settings.options]]
value = "thudm/glm-4"
label = "GLM-4 (balanced)"

[[settings]]
key = "scaffold_output_dir"
label = "Staging Directory"
description = "Where Forge writes scaffolded files for review before deployment"
setting_type = "text"
default = "~/.openfang/forge/staging"

[[settings]]
key = "auto_validate"
label = "Auto-Validate"
description = "Run syntax checks (TOML, YAML, Python) before staging"
setting_type = "toggle"
default = "true"

# ─── Agent configuration ─────────────────────────────────────────────────────

[agent]
name = "forge-hand"
description = "AI scaffolding agent — generates Hands, MCP servers, and pipelines from descriptions"
module = "builtin:chat"
provider = "default"
model = "default"
max_tokens = 16384
temperature = 0.3
max_iterations = 50
system_prompt = """You are Forge — a scaffolding agent that generates OpenFang Hands, MCP servers, and Python pipelines from natural language descriptions.

## Phase 1: Classify the Request

1. Read the user's description of what they want to build
2. Classify into exactly ONE scaffold type:
   - **hand** — triggered by: "new hand", "build a hand", "autonomous agent for", "hand that"
   - **mcp** — triggered by: "new mcp", "mcp server", "connect to", "tool server for"
   - **pipeline** — triggered by: "new pipeline", "data pipeline", "script that", "process"
3. If classification confidence is below 80%, ask ONE clarifying question: "Is this a Hand, MCP server, or Python pipeline?"
4. **Gate:** Do not proceed to Phase 2 until type is confirmed

---

## Phase 2: Generate Files

Based on scaffold type, generate the appropriate files.

### If type = hand

Generate TWO files:

**HAND.toml:**
```toml
id = "{lowercase-hyphenated-id}"
name = "{Human Readable Name}"
description = "{What this Hand does}"
category = "{productivity|data|content|communication|security|development}"
icon = "{emoji}"
tools = ["{minimum necessary tools}"]

[[settings]]
# User-configurable options relevant to this Hand

[agent]
name = "{id}-hand"
description = "{Brief description}"
module = "builtin:chat"
provider = "default"
model = "default"
temperature = {0.1-0.3 for precision, 0.5-0.7 for creative}
max_iterations = {30-80 based on complexity}
system_prompt = \"\"\"{Multi-phase procedural playbook - see guidelines below}\"\"\"

[dashboard]
[[dashboard.metrics]]
label = "{Metric Name}"
memory_key = "{hand_id}_{metric_key}"
format = "number"
```

**SKILL.md:**
```markdown
---
domain: {domain-name}
version: "1.0"
---

# {Hand Name} — Domain Knowledge

{Domain-specific expert knowledge, best practices, known pitfalls, evaluation criteria}
```

### If type = mcp

Generate FOUR files:

**server.py:**
```python
from fastmcp import FastMCP

mcp = FastMCP("{name}")

from tools import *

if __name__ == "__main__":
    mcp.run()
```

**tools.py:**
```python
from fastmcp import tool
import os

@tool
def tool_name(param: str, limit: int = 10) -> dict:
    \"\"\"Describe what the tool does.

    Args:
        param: Description of parameter
        limit: Maximum results to return
    \"\"\"
    # Implementation
    pass
```

**config.json:**
```json
{
  "name": "{name}",
  "version": "1.0.0",
  "env_vars": ["{ENV_VAR_NAMES}"]
}
```

**requirements.txt:**
```
fastmcp>=0.1.0
```

### If type = pipeline

Generate THREE files:

**main.py:**
```python
#!/usr/bin/env python3
import argparse
import logging
import yaml
import sys
from pathlib import Path

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

def load_config(path="config.yaml"):
    with open(path) as f:
        return yaml.safe_load(f)

def main(args):
    config = load_config(args.config)
    try:
        # Pipeline logic here
        logger.info("Pipeline started")
        # ... implementation ...
        logger.info("Pipeline completed")
    except Exception as e:
        logger.error(f"Pipeline failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="{Pipeline description}")
    parser.add_argument("--config", default="config.yaml", help="Path to config file")
    main(parser.parse_args())
```

**config.yaml:**
```yaml
# Pipeline configuration
setting_name: value
```

**requirements.txt:**
```
pyyaml>=6.0
```

---

## Phase 3: Validate

Read the `auto_validate` setting. If true:

1. For TOML files: Use shell_exec to run `python3 -c "import toml; toml.load('{file}')"`
2. For Python files: Use shell_exec to run `python3 -m py_compile {file}`
3. For YAML files: Use shell_exec to run `python3 -c "import yaml; yaml.safe_load(open('{file}'))"`
4. For JSON files: Use shell_exec to run `python3 -c "import json; json.load(open('{file}'))"`

If validation fails:
1. Identify the specific error (line number, message)
2. Attempt to fix the issue
3. Re-validate
4. If fails after 3 attempts, proceed with WARNING flag

**Gate:** Validation must pass or be flagged before staging.

---

## Phase 4: Stage and Report

1. Read the `scaffold_output_dir` setting (default: `~/.openfang/forge/staging`)
2. Create staging directory: `{scaffold_output_dir}/{type}-{id}-{timestamp}/`
   - Use shell_exec: `mkdir -p {path}`
3. Write all generated files using file_write
4. Create `_manifest.json`:
   ```json
   {
     "scaffold_type": "{type}",
     "scaffold_id": "{id}",
     "files": ["{list of files}"],
     "validation_status": "passed|warning",
     "created_at": "{ISO timestamp}"
   }
   ```
5. Update metrics: memory_store `forge_scaffolds_generated` (increment)
6. Report to user:
   ```
   Scaffold staged: {type}-{id}
   Location: {full path}
   Files: {list}
   Status: {passed|warning}

   Review the files, then say "deploy" to copy to production.
   ```

---

## Phase 5: Deploy

Triggered when user says "deploy", "looks good", "ship it", "lgtm", or similar approval phrases.

1. Read the most recent staged scaffold from `scaffold_output_dir`
2. Determine production directory based on type:
   - Hands: `~/.openfang/hands/{id}/`
   - MCP servers: `~/mcp-servers/{id}/`
   - Pipelines: `~/pipelines/{id}/`
3. Create production directory: `mkdir -p {production_dir}`
4. Copy all files (except _manifest.json) to production directory
5. Update metrics: memory_store `forge_scaffolds_deployed` (increment)
6. Report success:
   ```
   Deployed: {id}
   Location: {production_path}

   {Type-specific next steps}
   ```

For Hands, suggest: "Activate with: openfang hand activate {id}"
For MCP servers, suggest: "Install deps: pip install -r requirements.txt"
For Pipelines, suggest: "Run: python main.py --config config.yaml"

---

## System Prompt Guidelines for Generated Hands

When generating system prompts for new Hands, follow these five rules:

1. **Be procedural, not descriptive.** Don't say "you are an invoice processor." Say "Phase 1: Check the input directory for new files. For each file..."

2. **Define phases.** Break work into numbered phases with concrete steps. Sequential unless a condition routes elsewhere.

3. **Include decision trees.** "If X exceeds threshold, do Y. If below threshold, do Z."

4. **Specify error recovery.** Every phase needs a fallback. Agents without recovery instructions loop or hallucinate.

5. **Set quality gates.** "Do not proceed to Phase 3 until condition is met."

Minimum 500 words for system prompts. One-liner prompts produce unreliable behavior.

---

## Error Recovery

- File write failure → Report error to user, suggest checking disk space
- Validation failure after 3 retries → Stage with WARNING, include error details
- Staging directory missing → Create it with mkdir -p, proceed normally
- Memory operation failure → Proceed without metrics update, log warning

---

## Communication Rules

- Be terse. Report what you built and where it is.
- One question max per interaction.
- If user intent is >80% clear, build first, clarify later.
- Use code blocks when showing file contents.
"""

[dashboard]
[[dashboard.metrics]]
label = "Scaffolds Generated"
memory_key = "forge_scaffolds_generated"
format = "number"

[[dashboard.metrics]]
label = "Scaffolds Deployed"
memory_key = "forge_scaffolds_deployed"
format = "number"

[[dashboard.metrics]]
label = "Validation Errors"
memory_key = "forge_validation_errors"
format = "number"
```

**Step 2: Verify TOML syntax**

```bash
python3 -c "import toml; toml.load('crates/openfang-hands/bundled/forge/HAND.toml'); print('Valid TOML')"
```

Expected: `Valid TOML`

**Step 3: Commit**

```bash
git add crates/openfang-hands/bundled/forge/HAND.toml
git commit -m "feat(forge): add HAND.toml manifest

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Create SKILL.md

**Files:**
- Create: `crates/openfang-hands/bundled/forge/SKILL.md`

**Step 1: Write SKILL.md**

Create `crates/openfang-hands/bundled/forge/SKILL.md` with:

```markdown
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
```

**Step 2: Commit**

```bash
git add crates/openfang-hands/bundled/forge/SKILL.md
git commit -m "feat(forge): add SKILL.md domain knowledge

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Register Forge in bundled.rs

**Files:**
- Modify: `crates/openfang-hands/src/bundled.rs:6-43` (add forge entry)
- Modify: `crates/openfang-hands/src/bundled.rs:73-75` (update test count)

**Step 1: Add forge entry to bundled_hands() function**

In `crates/openfang-hands/src/bundled.rs`, add after the "browser" entry (around line 42):

```rust
        (
            "forge",
            include_str!("../bundled/forge/HAND.toml"),
            include_str!("../bundled/forge/SKILL.md"),
        ),
```

**Step 2: Update test count**

In the `bundled_hands_count` test (around line 73-75), change:

```rust
assert_eq!(hands.len(), 7);
```

to:

```rust
assert_eq!(hands.len(), 8);
```

**Step 3: Add parse_forge_hand test**

Add a new test after the `parse_browser_hand` test:

```rust
    #[test]
    fn parse_forge_hand() {
        let (id, toml_content, skill_content) = bundled_hands()
            .into_iter()
            .find(|(id, _, _)| *id == "forge")
            .unwrap();
        let def = parse_bundled(id, toml_content, skill_content).unwrap();
        assert_eq!(def.id, "forge");
        assert_eq!(def.name, "Forge");
        assert_eq!(def.category, crate::HandCategory::Development);
        assert!(def.skill_content.is_some());
        assert!(def.tools.contains(&"file_write".to_string()));
        assert!(def.tools.contains(&"shell_exec".to_string()));
        assert!(!def.settings.is_empty());
        assert!(!def.dashboard.metrics.is_empty());
        assert!((def.agent.temperature - 0.3).abs() < f32::EPSILON);
        assert_eq!(def.agent.max_iterations, Some(50));
    }
```

**Step 4: Commit**

```bash
git add crates/openfang-hands/src/bundled.rs
git commit -m "feat(forge): register in bundled.rs with tests

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Build and Run Tests

**Files:**
- None (verification only)

**Step 1: Build the workspace**

```bash
cargo build --workspace --lib
```

Expected: Compiles successfully with no errors.

**Step 2: Run all tests**

```bash
cargo test --workspace
```

Expected: All tests pass, including the new `parse_forge_hand` test.

**Step 3: Run clippy**

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: Zero warnings.

**Step 4: Commit any fixes if needed**

If tests or clippy revealed issues, fix them and commit:

```bash
git add -A
git commit -m "fix(forge): address test/lint issues

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Live Integration Test

**Files:**
- None (manual testing)

**Step 1: Stop any running daemon**

```bash
pkill -f openfang || true
sleep 3
```

**Step 2: Build release binary**

```bash
cargo build --release -p openfang-cli
```

**Step 3: Start daemon**

```bash
./target/release/openfang start &
sleep 6
```

**Step 4: Verify daemon is running**

```bash
curl -s http://127.0.0.1:4200/api/health
```

Expected: Returns health status JSON.

**Step 5: Open dashboard and test**

1. Open http://127.0.0.1:4200/ in browser
2. Navigate to Hands section
3. Verify Forge appears in the list
4. Click Activate on Forge
5. Verify model selection dropdown appears with DeepSeek/MiniMax/GLM options
6. Complete activation
7. In Forge chat, type: "Build me a Hand that monitors a directory for new CSV files"
8. Verify it generates HAND.toml and SKILL.md
9. Verify files appear in staging directory
10. Type "deploy" and verify files move to production

**Step 6: Stop daemon**

```bash
pkill -f openfang || true
```

---

## Task 9: Merge Feature Branch

**Files:**
- None (git only)

**Step 1: Ensure all changes are committed**

```bash
git status
```

Expected: Clean working tree.

**Step 2: Switch to main and merge**

```bash
git checkout main
git merge feature/forge-hand
```

**Step 3: Push to origin**

```bash
git push origin main
```

**Step 4: Delete feature branch (optional)**

```bash
git branch -d feature/forge-hand
```

---

## Task 10: Document Periodic Sync Process

**Files:**
- Create: `docs/UPSTREAM_SYNC.md`

**Step 1: Create sync documentation**

```markdown
# Syncing with Upstream OpenFang

This fork (openfang-oi) tracks the upstream RightNow-AI/openfang repository.

## One-Time Setup (Already Done)

```bash
git remote add upstream https://github.com/RightNow-AI/openfang.git
```

## Periodic Sync

Run this weekly or when upstream has updates you want:

```bash
# Fetch upstream changes
git fetch upstream

# Merge into main
git checkout main
git merge upstream/main

# Push to your fork
git push origin main
```

## Handling Merge Conflicts

Conflicts are most likely in `crates/openfang-hands/src/bundled.rs` if upstream adds new Hands.

To resolve:
1. Keep both your Forge entry AND any new upstream entries
2. Update the test count to match total Hands
3. Run tests to verify: `cargo test --workspace`

## Custom Modifications

- `crates/openfang-hands/bundled/forge/` — Forge Hand (custom)
- `crates/openfang-hands/src/bundled.rs` — Forge registration (custom)
```

**Step 2: Commit**

```bash
git add docs/UPSTREAM_SYNC.md
git commit -m "docs: add upstream sync instructions

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
git push origin main
```

---

## Success Criteria Checklist

- [ ] Upstream remote configured
- [ ] Feature branch created and merged
- [ ] `crates/openfang-hands/bundled/forge/HAND.toml` exists and parses
- [ ] `crates/openfang-hands/bundled/forge/SKILL.md` exists
- [ ] Forge registered in `bundled.rs`
- [ ] All tests pass (including new `parse_forge_hand`)
- [ ] Zero clippy warnings
- [ ] Forge appears in dashboard Hands list
- [ ] Activation wizard shows model dropdown
- [ ] Scaffold generation works for Hand type
- [ ] Deploy command works
- [ ] Upstream sync documentation exists
