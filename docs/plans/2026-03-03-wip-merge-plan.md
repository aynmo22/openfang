# WIP Merge Plan — Fix Build Errors

**Created:** 2026-03-03
**Status:** In Progress
**Context:** Merging upstream v0.3.10 + local WIP customizations

---

## Current State

- **Branch:** `main`
- **Upstream merged:** v0.3.10 (23 commits ahead of origin)
- **WIP changes staged:** All conflicts resolved, but **18 build errors**
- **Stash:** Empty (already popped)

## What's in WIP (valuable work)

| Feature | Files | Lines |
|---------|-------|-------|
| Scraper module | `scraper.rs`, `scraper_bridge.py` | ~560 |
| Scraper tools | `tool_runner.rs` | +47 |
| OpenRouter helpers | `routes.rs` | ~500 net |
| Agent configs | 30 `agent.toml` files | openrouter/glm-5 |
| Runtime tweaks | `agent_loop.rs`, `consolidation.rs`, `openai.rs` | ~400 |
| Forge Hand | Already committed | ✅ Done |

## Build Errors to Fix (18 total)

**Root cause:** `server.rs` references route handlers that don't exist in WIP's `routes.rs`.

**Actual errors:**
```
E0425: cannot find value `clear_agent_history` in module `routes`
E0425: cannot find value `get_agent_tools` in module `routes`
E0425: cannot find value `set_agent_tools` in module `routes`
E0425: cannot find value `clawhub_skill_code` in module `routes`
E0425: cannot find value `install_hand` in module `routes`
E0425: cannot find value `get_hand_settings` in module `routes`
E0425: cannot find value `update_hand_settings` in module `routes`
E0425: cannot find value `comms_topology` in module `routes`
E0425: cannot find value `comms_events` in module `routes`
E0425: cannot find value `comms_events_stream` in module `routes`
E0425: cannot find value `comms_send` in module `routes`
E0425: cannot find value `comms_task` in module `routes`
E0425: cannot find value `update_agent_budget` in module `routes`
E0425: cannot find value `add_custom_model` in module `routes`
E0425: cannot find value `remove_custom_model` in module `routes`
E0425: cannot find value `copilot_oauth_start` in module `routes`
E0425: cannot find value `copilot_oauth_poll` in module `routes`
E0560: struct `AppState` has no field named `clawhub_cache`
```

**What happened:** WIP's `routes.rs` is missing functions that `server.rs` expects. Either:
1. WIP deleted them accidentally during conflict resolution
2. Or they need to be copied from upstream's routes.rs

## Fix Strategy (Fastest Path)

**The problem:** We took WIP's routes.rs, but it's missing 17 route handlers that server.rs needs.

**Fastest fix:** Take UPSTREAM's routes.rs, then ADD your OpenRouter helpers to it.

### Task 0: Fix routes.rs (the main issue)

```bash
# Get upstream's complete routes.rs
git show upstream/main:crates/openfang-api/src/routes.rs > /tmp/upstream-routes.rs

# Compare to see what WIP added (OpenRouter helpers around line 5054)
diff -u /tmp/upstream-routes.rs crates/openfang-api/src/routes.rs | head -200

# Option A: Start with upstream, add WIP's OpenRouter code
cp /tmp/upstream-routes.rs crates/openfang-api/src/routes.rs
# Then manually add the OpenRouter helpers from WIP

# Option B: Or just use upstream's routes.rs and add OpenRouter later
git checkout upstream/main -- crates/openfang-api/src/routes.rs
```

### Task 1: Identify all missing fields/types

```bash
cargo build --workspace --lib 2>&1 | grep "error\[" | sort | uniq -c | sort -rn
```

### Task 2: Add missing fields to AppState

Look at `crates/openfang-api/src/server.rs` — the `AppState` struct needs fields that WIP code expects:
- `clawhub_cache`
- Others (check errors)

### Task 3: Add missing imports

Check for `E0425` errors (cannot find value/function) — add missing `use` statements.

### Task 4: Reconcile type mismatches

Check for `E0308` errors (mismatched types) — may need to update function signatures.

### Task 5: Build and test

```bash
cargo build --workspace --lib
cargo test -p openfang-hands --lib  # Verify Forge still works
cargo test --workspace              # Full test suite
```

### Task 6: Commit WIP

```bash
git commit -m "feat: add scraper module, OpenRouter helpers, agent configs

- Add scrape_url and scrape_dynamic tools with Python bridge
- Add OpenRouter custom model sync helpers
- Configure all agents to use openrouter/z-ai/glm-5
- Add memory consolidation improvements

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

### Task 7: Push and prepare release

```bash
git push origin main
```

Then proceed with release:
1. Add GitHub secrets (TAURI_SIGNING_PRIVATE_KEY)
2. Tag v0.3.11
3. Set up openfang.sh domain

---

## Alternative: Discard WIP, Ship Clean

If fixing errors takes too long:

```bash
# Reset to clean state (keeps Forge, discards WIP)
git reset --hard ac0f5d0   # The upstream merge commit
git push origin main --force-with-lease

# Re-apply agent configs only (easy)
# Script to update all agents to openrouter/glm-5
```

---

## Files Reference

**Already committed (safe):**
- `crates/openfang-hands/bundled/forge/` — Forge Hand ✅
- `docs/UPSTREAM_SYNC.md` — Sync instructions ✅

**Staged WIP (needs build fix):**
- `crates/openfang-runtime/src/scraper.rs` — NEW
- `crates/openfang-runtime/src/scraper_bridge.py` — NEW
- `crates/openfang-api/src/routes.rs` — Modified
- `crates/openfang-runtime/src/tool_runner.rs` — Modified
- 30x `agents/*/agent.toml` — Modified

---

## Quick Start for New Session

```
Load this plan: docs/plans/2026-03-03-wip-merge-plan.md

Current status:
- Upstream v0.3.10 merged
- WIP changes staged but won't compile (18 errors)
- Need to fix build errors in server.rs and related files

Run: cargo build --workspace --lib 2>&1 | head -100
Then fix each error systematically.
```
