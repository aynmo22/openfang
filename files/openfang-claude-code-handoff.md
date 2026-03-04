# OpenFang Productization — Claude Code Handoff

## Who This Is For

Al, Director of Product & Technology at Open Influence. This document is a handoff from a strategic planning + initial build session in Claude.ai. Claude Code should use this as the complete context to continue building.

---

## The Big Picture

Al has a running instance of **OpenFang** (v0.1.0), an open-source Agent Operating System built in Rust. It's forked at `openfang-oi`. He's using **OpenRouter** as his LLM provider. The goal is to productize OpenFang for his personal use — building autonomous agents that 10x his productivity as a technical leader who wears too many hats.

Al is NOT building tools for his team or clients with this. He's building tools **for himself** — the person who builds everything, holds all context, and is the single point of technical failure at a $75M company.

---

## The 5 Opportunities (Prioritized Build Order)

### 1. Forge — The Builder's Forge
**Purpose:** Personal code generation and scaffolding agent. Al describes what he needs, Forge generates production-ready scaffolds, stages them for review, and deploys on approval.

**What it scaffolds:**
- OpenFang Hands (HAND.toml + system-prompt.md + SKILL.md)
- MCP servers (Python/FastMCP — server.py, tools.py, config.json, requirements.txt)
- Python data pipelines (main.py, config.yaml, requirements.txt)

**Interaction model:**
- Primary: Slack command + OpenFang dashboard chat
- Stretch: Natural language description from anywhere

**Autonomy level:** Generates and stages files → Al reviews → one-click deploy to production

**LLM routing (via OpenRouter, NEVER Claude API):**
- Code/config generation: DeepSeek V3.2 (`deepseek/deepseek-chat`)
- Long-form content (system prompts, SKILL.md): Kimi K2.5 (`moonshot/moonshot-v1-128k`) or MiniMax M2.5 (`minimax/minimax-01`)
- General purpose: GLM-5 (`thudm/glm-4`)

**Status: BUILD THIS FIRST. Files drafted but need to be rebuilt against the actual CLI/dashboard (see Current State below).**

### 2. Mission Control — Autonomous Chief of Staff
**Purpose:** Not a status dashboard — a living chief of staff that captures Al's full inbound work stream and manages his queue.

**Inputs it needs to capture:**
- Slack requests and threads (people asking Al for things)
- JIRA tickets assigned or relevant to him
- Granola meeting transcripts (action items, decisions)
- Google Sheets activity (what Al worked on, reviewed, edited — this gets lost daily)
- Calendar (time allocation, conflicts, CEO meetings)

**What it does:**
- Maintains a living task queue across all sources
- Tracks what Al actually did each day (so he stops losing track)
- Flags what's slipping or stalled
- Prepares strategic context and talking points for CEO conversations
- Delivers morning briefing to Slack + dashboard

**This is the one that solves Al's #1 pain: context-switching across a dozen concurrent projects.**

### 3. Amplifier — Strategic Leverage Agent
**Purpose:** Makes Al bigger than one person. Bridges the gap between what he does and how he's perceived.

**What it does:**
- Prepares CEO meeting talking points (pulling from Mission Control's context)
- Drafts communications in Al's voice
- Curates learning paths for skills Al needs next
- Builds personal brand positioning — the narrative around accomplishments and trajectory

**Why it's #3:** Al selected all four sub-options (CEO prep, personal brand, communication drafts, learning) — they're all facets of being strategically underlevered for the scope he owns.

### 4. AI Radar — Strategic Intelligence Engine
**Purpose:** Al's research team that never sleeps.

**What it monitors:**
- AI/ML landscape (new models, frameworks, techniques — especially on OpenRouter)
- Creator economy shifts
- Competitor moves (CreatorIQ, Grin, Captiv8, etc.)
- Client industry trends

**Key differentiator from a news feed:** It recommends actions, not just information. Feeds directly into product roadmap decisions.

### 5. Boundaries — The Time Guardian
**Purpose:** Autonomous scheduling agent that enforces work/life boundaries.

**What it does:**
- Reads calendar and Slack activity patterns
- Detects when work is overrunning personal time
- Protects time blocks for personal activities (surfing, jiu jitsu, maker projects)
- Active time-blocking, not passive nudging

**Why it's last:** Depends on Mission Control being in place to know what's actually on Al's plate.

---

## Current State of OpenFang

### Version & Installation
- **Version:** v0.1.0 (shown in dashboard header)
- **Platform:** macOS (Al's MacBook Pro)
- **Fork:** `openfang-oi` (Al's fork of RightNow-AI/openfang)
- **Daemon:** Running, dashboard accessible at `http://127.0.0.1:4200/`
- **Running agents:** 2 — "General Assistant" and "researcher-hand", both using `z-ai/glm-5` via OpenRouter

### What Works
- Dashboard is up and rendering
- Agent templates are available (General Assistant, Code Helper, Researcher, Writer, Data Analyst, DevOps Engineer, Customer Support, Tutor, API Designer, Meeting Notes)
- Hands page shows all 7 bundled Hands (Browser, Clip, Collector, Lead, Predictor, Researcher, Twitter)
- Left sidebar navigation: Chat, Overview, Analytics, Logs, Sessions, Approvals, Workflows, Scheduler, Channels, Skills, Hands, Settings

### What Doesn't Work (Known Issues)
- **Hand Activate button does nothing.** Clicking "Activate" on any Hand in the dashboard (e.g., Researcher Hand) produces no response — no error, no state change, nothing. This needs debugging. Check browser console for JS errors and `openfang logs` for backend errors when the button is clicked.
- **No `hand` CLI subcommand.** Running `openfang hand validate` returns "unrecognized subcommand 'hand'". The Hands documentation at openfang.sh/docs/hands describes a `hand` subcommand (activate, deactivate, status, pause, resume, install, validate, pack, publish) that does not exist in v0.1.0. The docs are ahead of the code.
- **Model ID may be wrong.** Dashboard shows `z-ai/glm-5` as the model — this may need to be `zhipu-ai/glm-4` or similar valid OpenRouter model ID. Verify against OpenRouter's model catalog.

### Available CLI Commands (v0.1.0)
These are the ACTUAL commands available (not what the docs say):

```
init, start, stop, agent, workflow, trigger, migrate, skill, channel, config,
chat, status, doctor, dashboard, completion, mcp, add, remove, integrations,
vault, new, tui, models, gateway, approvals, cron, sessions, logs, health,
security, memory, devices, qr, webhooks, onboard, setup, configure, message,
system, reset
```

Key commands with subcommands (marked [*]): `agent`, `workflow`, `trigger`, `skill`, `channel`, `config`, `vault`, `models`, `gateway`, `approvals`, `cron`, `sessions`, `security`, `memory`, `devices`, `webhooks`, `system`

**IMPORTANT: Need to run `openfang agent --help`, `openfang skill --help`, `openfang new --help`, and `openfang cron --help` to understand the actual primitives available for building Forge.**

### Hand Architecture (from official docs)
Each Hand is a 3-layer package:

1. **HAND.toml** — Declarative manifest
   - `[hand]` — id, name, description, category, icon, version
   - `[hand.requires]` — tools list
   - `[hand.settings]` — user-configurable options (types: string, bool, int, float, string[])
   - `[hand.agent]` — model, temperature, max_iterations
   - `[hand.schedule]` — cron expression
   - `[hand.dashboard]` — metrics array
   - `[hand.triggers]` — event-based activation (optional)
   - `[hand.chain]` — multi-Hand orchestration (optional)

2. **system-prompt.md** — Multi-phase operational playbook
   - Must be procedural, not descriptive
   - Numbered phases with concrete steps
   - Decision trees, error recovery, quality gates
   - Minimum 500 words

3. **SKILL.md** — Domain knowledge with YAML frontmatter
   - Reference material, not instructions
   - Best practices, pitfalls, evaluation criteria

Bundled Hands are compiled into the binary via Rust's `include_str!()`. Custom Hands installed at runtime load from `~/.openfang/hands/`.

Installation flow (per docs — may not work in v0.1.0):
```bash
openfang hand validate ./my-hand/
openfang hand install ./my-hand/
openfang hand activate my-hand
```

---

## Two Paths Forward for Forge

### Path A: Agent Route (Fast)
Build Forge as a regular **agent** using existing CLI primitives:
- `openfang agent new` — create the agent
- `openfang skill` — install domain knowledge
- `openfang cron` — schedule maintenance runs
- Chat with Forge via dashboard or `openfang chat`

**Pros:** Works with what's available today. No Rust compilation needed.
**Cons:** No Hand-specific features (dashboard metrics card, lifecycle management, built-in approval gates).

### Path B: Proper Hand in the Fork (Correct)
Add Forge as a custom Hand in the `openfang-oi` fork source code:
- Add `forge/` directory alongside the 7 existing Hands in the codebase
- Add HAND.toml, system-prompt.md, SKILL.md
- Recompile the Rust binary

**Pros:** Full Hand experience — metrics, lifecycle, dashboard card, proper architecture.
**Cons:** Requires Rust build of 137K LOC codebase. Need to understand the Hand registration code.

### Recommended Approach
**Start with Path A to get something working immediately**, then migrate to Path B once the Hand activation bug is fixed and the codebase is understood. The system prompt and SKILL.md content are identical either way — only the packaging differs.

---

## Immediate Next Steps for Claude Code

### Priority 1: Debug Hand Activation
The Activate button in the Hands dashboard doesn't work. This blocks everything.
1. Open browser dev tools → Console → click Activate → capture errors
2. Run `openfang logs` → click Activate → capture backend output
3. Check if this is an API auth issue (the release notes mention "Dashboard approve/reject buttons returning 401" was fixed in a patch)
4. Check the model ID `z-ai/glm-5` — may need to be corrected in config

### Priority 2: Explore Actual Agent Primitives
Run these and understand what's available:
```bash
openfang agent --help
openfang skill --help
openfang new --help
openfang cron --help
openfang workflow --help
```

### Priority 3: Build Forge (Agent Route)
Using whatever primitives are available, create the Forge agent with:
- The system prompt from the drafted files (see artifacts below)
- The SKILL.md domain knowledge
- A cron schedule for daily maintenance
- Slack channel integration for notifications

### Priority 4: Build Forge (Hand Route)
Once Hand activation is fixed:
- Locate the Hands directory in the fork's source code
- Study how existing Hands (especially Researcher) are structured
- Add Forge as a custom Hand
- Recompile and test

---

## Al's Technical Preferences & Constraints

- **LLM routing:** ALWAYS use OpenRouter. NEVER use Claude API for workflows. Claude is used ONLY via Claude Code.
- **Preferred OpenRouter models:** DeepSeek V3.2, MiniMax M2.5, GLM-5, Kimi K2.5
- **Orchestration:** N8N for workflow automation (uses OpenRouter models)
- **Infrastructure:** EC2 instances, Elasticsearch (~2M creator records, NOT 5.8B)
- **Communication:** Slack primary
- **Approach:** Speed and reliability over cost. Direct, actionable over theoretical.
- **Creator Search MCP:** Needs deployment to EC2 in same VPC as Elasticsearch for 24/7 access

---

## Drafted Forge Files

The following files were created during this session. They match the official HAND.toml schema from the docs but need to be adapted to whatever primitives actually work in v0.1.0.

### Files Location
The drafted files should be available at:
- `HAND.toml` — manifest matching `[hand.*]` schema
- `system-prompt.md` — 7-phase operational playbook
- `SKILL.md` — domain knowledge with YAML frontmatter
- `README.md` — deployment guide

### Key Content in system-prompt.md
7 phases: Classify Request → Check Pattern Library → Generate Files → Validate → Stage and Report → Deploy (on approval) → Learn from deploys

### Key Content in SKILL.md
- OpenFang Hand structure reference
- Available tools list
- OpenRouter model reference with strengths/use cases
- MCP server scaffolding patterns (FastMCP)
- Python pipeline scaffolding patterns
- System prompt design principles (5 rules from OpenFang docs)
- Al's environment context
- Known pitfalls

---

## Context About Al

- **Role:** Director of Product & Technology at Open Influence ($75M creator marketing, Fortune 1000 clients)
- **Team:** 7-person tech team, reports to CEO with full budget authority
- **Reality:** Wearing Director, de facto CTO, IT security, developer, and data engineer hats simultaneously
- **#1 pain:** Context-switching across a dozen concurrent projects
- **#2 pain:** Work overrunning personal life (surfing, jiu jitsu, maker projects get pushed)
- **Active projects:** Creator Search MCP, RFI Agent, N8N workflows, sentiment analysis, UGC pipeline, brand safety tools
- **Personal projects:** Solar installations, lawn renovation, 3D printing, garage AI assistant
