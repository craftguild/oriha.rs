<!-- craftguild:common:start -->
Use Craftguild standards 0.1.0 with the fixed engine in standards/engine.json.
Run `make prime` (or `python3 standards/launcher.py prime`) before adoption.
Read relevant topics with `make prime PRIME_ARGS="--topic NAME"`.
Follow the repo-local craftguild-standardize Skill. Run `make check`.
Update explicitly with `craftguild-standards init [--force]`; follow its dependency revision guidance.
<!-- craftguild:common:end -->

## Shared agent files

`AGENTS.md` is the repository instruction authority. The two shared skills in
`.agents/skills/` cover Beads and the pinned Craftguild entry. Tool-specific
`.claude/`, `.codex/`, `.cursor/` and editor settings are local and ignored.
Beads data and configuration stay entirely local in ignored `.beads/`; do not
publish or synchronize them. Run `bd prime` for the local task workflow.

## Repository contracts

Read [the documentation index](docs/README.md), accepted
[ADRs](docs/adr/README.md), and the relevant current design before changing behavior.
Keep Markdown parsing, source mapping, block derivation, typed native contracts,
resource limits, escaping and diagnostics unchanged during standardization.
Preserve JSON snapshots and the 652-case CommonMark profile contract. Never accept
snapshot or generated-binding changes merely to make a check pass.

CLI artifact acceptance scenarios and assertions belong in Rust integration
tests. `make acceptance` runs the opt-in test against the verified built CLI;
scripts only prepare its environment and invoke commands. Keep the test build
directory separate from the shipped artifact. Other components and adopted
frameworks retain their own test language and tooling.

Canonical tables and bridge outputs must follow their actual generators and
ownership/provenance checks. Keep first-party and third-party rights distinct.
Do not hand-edit generated output or discard required fixtures and goldens while
preparing a public inventory. Original evaluation records belong in ignored
private archives and must not become build/test inputs.

Use [Beads](.agents/skills/beads/SKILL.md) for durable work. Run `bd prime`, inspect
the issue and claim before editing. Close only completed work. Do not create a
second Markdown task/review ledger.

Preserve unrelated changes, native state and credentials. Tests use owned,
isolated resources. Installation, publication, Git staging/commits/pushes and
service operations require active user authorization; tooling introduction does
not grant it.
