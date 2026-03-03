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
- `crates/openfang-hands/src/registry.rs` — Updated test count (custom)
