# Forge Hand Design

**Date:** 2026-03-02
**Author:** Al (with Claude Code)
**Status:** Approved

## Overview

Forge is a bundled Hand for OpenFang that generates scaffolds for three artifact types:

1. **OpenFang Hands** — HAND.toml + SKILL.md (system prompt inline)
2. **MCP servers** — server.py + tools.py + config.json + requirements.txt (FastMCP)
3. **Python pipelines** — main.py + config.yaml + requirements.txt

### Interaction Model

Forge runs as its own agent with a dedicated chat interface. Users open the Forge Hand chat when they want to scaffold something. It does not monitor other chats or agents.

```
User describes what they need (Forge chat)
    ↓
Forge classifies request type (hand/mcp/pipeline)
    ↓
Forge generates files to staging directory
    ↓
User reviews staged files
    ↓
User says "deploy" → Forge copies to production directory
```

### Scope (v1)

**Included:**
- Scaffold generation for hands, MCP servers, and pipelines
- Syntax validation (TOML, YAML, JSON, Python)
- Staging and review flow
- Deploy via chat command
- User-selectable generation model

**Deferred to v2:**
- Slack notifications
- Pattern learning from deployed scaffolds
- Model routing (different models for different file types)

## Architecture

### File Location

```
crates/openfang-hands/bundled/forge/
├── HAND.toml    # Manifest (flat format matching other bundled Hands)
└── SKILL.md     # Domain knowledge for scaffolding
```

### Registration

Add entry to `crates/openfang-hands/src/bundled.rs`:
```rust
(
    "forge",
    include_str!("../bundled/forge/HAND.toml"),
    include_str!("../bundled/forge/SKILL.md"),
),
```

Update test count from 7 to 8 in `bundled_hands_count` test.

## HAND.toml Structure

```toml
id = "forge"
name = "Forge"
description = "Personal builder's scaffolding agent. Generates OpenFang Hands, MCP servers, and Python pipelines from natural language descriptions."
category = "development"
icon = "🔨"
tools = [
    "file_read", "file_write", "file_list", "file_delete",
    "shell_exec", "memory_store", "memory_recall"
]

# --- Settings ---

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
description = "Where Forge writes scaffolded files for review"
setting_type = "text"
default = "~/.openfang/forge/staging"

[[settings]]
key = "auto_validate"
label = "Auto-Validate"
description = "Run syntax checks (TOML, YAML, Python) before staging"
setting_type = "toggle"
default = "true"

# --- Agent Configuration ---

[agent]
name = "forge-hand"
description = "AI scaffolding agent — generates Hands, MCP servers, and pipelines from descriptions"
module = "builtin:chat"
provider = "default"
model = "default"
max_tokens = 16384
temperature = 0.3
max_iterations = 50
system_prompt = """..."""  # See Operational Flow section

# --- Dashboard Metrics ---

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

## Operational Flow (System Prompt)

### Phase 1: Classify Request

1. Parse user's description
2. Classify as: `hand` | `mcp` | `pipeline`
3. Classification triggers:
   - **hand** — "new hand", "build a hand", "autonomous agent for", "hand that"
   - **mcp** — "new mcp", "mcp server", "connect to", "tool server for"
   - **pipeline** — "new pipeline", "data pipeline", "script that", "process"
4. If confidence < 80%, ask ONE clarifying question
5. **Gate:** Do not proceed until type is confirmed

### Phase 2: Generate Files

Based on scaffold type, generate:

**Hand:**
- `HAND.toml` — Full manifest with id, name, description, category, tools, settings, agent config (system_prompt inline), dashboard metrics
- `SKILL.md` — Domain knowledge with YAML frontmatter

**MCP Server:**
- `server.py` — FastMCP server with tool registration
- `tools.py` — Tool implementations with typed parameters
- `config.json` — Connection config, env var references (never hardcoded secrets)
- `requirements.txt` — Pinned dependencies

**Pipeline:**
- `main.py` — Entry point with argparse, logging, config loader
- `config.yaml` — Externalized configuration
- `requirements.txt` — Pinned dependencies

### Phase 3: Validate

If `auto_validate` setting is true:

1. TOML files: parse with TOML parser, must produce zero errors
2. Python files: run `python -m py_compile {file}`, must exit 0
3. YAML files: parse with PyYAML, must produce zero errors
4. JSON files: parse with json module, must produce zero errors

If validation fails:
1. Identify specific error (line number, message)
2. Attempt auto-fix
3. Re-validate
4. If fails after 3 attempts, stage with WARNING flag

**Gate:** Validation must pass or be flagged before staging.

### Phase 4: Stage

1. Create directory: `{scaffold_output_dir}/{type}-{id}-{timestamp}/`
2. Write all generated files
3. Create `_manifest.json`:
   ```json
   {
     "scaffold_type": "hand",
     "scaffold_id": "my-new-hand",
     "files": ["HAND.toml", "SKILL.md"],
     "validation_status": "passed",
     "created_at": "2026-03-02T12:00:00Z"
   }
   ```
4. Report to user: list files created, say "Review files or say 'deploy'"
5. Increment `forge_scaffolds_generated` metric

### Phase 5: Deploy

Triggered when user says "deploy", "looks good", "ship it", or similar.

1. Read staged files from most recent scaffold
2. Copy to production directory:
   - Hands: `~/.openfang/hands/{id}/`
   - MCP servers: `~/mcp-servers/{id}/`
   - Pipelines: `~/pipelines/{id}/`
3. Report success with deployment path
4. Increment `forge_scaffolds_deployed` metric

### Communication Rules

- Be terse. Report what you built and where it is.
- One question max per interaction.
- If intent is >80% clear, build first, clarify later.

### Error Recovery

- File write failure → check disk space, alert user, do not retry silently
- Validation failure after 3 retries → stage with WARNING, include error details
- Staging directory missing → create it, proceed normally

## SKILL.md Content

Domain knowledge covering:

1. **OpenFang Hand Structure** — HAND.toml schema, setting types, tool list
2. **MCP Server Patterns** — FastMCP structure, tool definition template
3. **Pipeline Patterns** — Entry point template, config loading
4. **System Prompt Design** — The five rules (procedural, phases, decision trees, error recovery, quality gates)
5. **Known Pitfalls** — Common mistakes to avoid

## Git Workflow

### One-Time Setup

```bash
# Add upstream remote
git remote add upstream https://github.com/RightNow-AI/openfang.git
git fetch upstream

# Create feature branch for Forge
git checkout -b feature/forge-hand
```

### Periodic Sync (Pull Upstream Updates)

```bash
git checkout main
git fetch upstream
git merge upstream/main
git push origin main

# Rebase feature branch if needed
git checkout feature/forge-hand
git rebase main
```

### Merge Conflict Strategy

Forge adds new files and one entry to `bundled.rs`. Conflicts are unlikely but if they occur in `bundled.rs`, manually merge the new Hand entries.

## Build & Test

```bash
# Compile
cargo build --workspace --lib

# Run tests (count increases from 1744+ to include Forge tests)
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets -- -D warnings
```

## Live Integration Test

1. Stop any running daemon
2. Build fresh: `cargo build --release -p openfang-cli`
3. Start daemon: `openfang start`
4. Open dashboard at http://127.0.0.1:4200/
5. Navigate to Hands → Forge → Activate
6. Test scaffold generation:
   - "Build me a Hand that monitors a directory for new files"
   - "Create an MCP server for querying Elasticsearch"
   - "Make a pipeline that processes CSV files"
7. Verify files appear in staging directory
8. Say "deploy" and verify files move to production directory
9. Check dashboard metrics update

## Success Criteria

- [ ] Forge appears in Hands list in dashboard
- [ ] Activation wizard shows model selection dropdown
- [ ] "Build me a Hand for X" generates valid HAND.toml + SKILL.md
- [ ] "Build me an MCP server for X" generates valid FastMCP scaffold
- [ ] "Build me a pipeline for X" generates valid Python scaffold
- [ ] Validation catches syntax errors before staging
- [ ] "deploy" command moves files to correct production directory
- [ ] Dashboard metrics increment correctly
- [ ] All cargo tests pass (including new Forge tests)
- [ ] Zero clippy warnings
