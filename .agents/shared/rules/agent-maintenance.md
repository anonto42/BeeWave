# Agent Knowledge Maintenance Rules

AI assistants (Antigravity, Gemini, Claude, Codex) MUST keep the `.agents/` directory synchronized with all project changes.

## 1. Automatic Update Requirement
Whenever an agent implements, refactors, or archives changes in `crates/`, `firmware/`, `web/`, or `openspec/`:
- The agent MUST review and update `.agents/shared/memory/` before completing the task.
- The agent MUST ensure `.agents/shared/agent-startup.md` accurately reflects current commands, tools, and workflows.

## 2. Memory Updates (`.agents/shared/memory/`)
- **`architecture.md`**: Record any new architectural decision, protocol format modification, memory/concurrency constraint, or storage design choice.
- **`capabilities.md`**: Record newly implemented domain capabilities, security detectors, anomaly rules, telemetry types, or client views.

## 3. Rule Updates (`.agents/shared/rules/`)
- If coding invariants, testing requirements, or git policies change, update the relevant file under `.agents/shared/rules/` (`rust-standards.md`, `file-structure.md`, `git.md`, `safety.md`).
- Never leave stale or conflicting guidelines in `.agents/shared/`.

## 4. OpenSpec Lifecycle Integration
- During the `apply` or `archive` stage of any OpenSpec change, summarize the durable architectural additions in `.agents/shared/memory/` so subsequent agent sessions immediately inherit context without scanning archived changes.
