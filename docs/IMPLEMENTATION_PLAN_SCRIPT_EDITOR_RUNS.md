# Implementation Plan: script editing and live runs

Status: implementation detail of the approved UI/functionality and operational-workflows plans (user “go”).

## Goal
Edit sources in an expandable offline code editor and run existing multi-step scenarios with live output, step progress and navigation during execution.

## Existing behavior
Native workers already survive navigation. Reports currently persist completed phases only; sources use a textarea. Existing schema: 1 check/shell tasks verify by repeating check. Upstream Ansible remains unchanged.

## Proposed approach / file changes
- `crates/ghosh-core/src/runs.rs`: optional executor lifecycle/stream hooks, backward-compatible report progress; keep existing check/apply/verify and cancellation semantics.
- `crates/ghosh-ssh/src/exec.rs`, `src-tauri/src/services/local_exec.rs`: drain both streams while publishing bounded output during execution.
- `src-tauri/src/services/runs.rs`: keep current progress in native memory, merge into snapshots; persist completed phases/final reports. Input masking handles split reads; snapshots never include credentials.
- `src/features/automation/RunsView.tsx`, `src/app/AppShell.tsx`, `src/app/App.tsx`: live snapshots, current step, progress, followable logs and background status; navigating does not cancel workers.
- `src/features/automation/examples.ts`: working multi-step example using the existing format.
- `src/features/automation/AutomationView.tsx`, new editor component, dependency/build configuration: expandable bundled editor, syntax/completion, offline diagnostics, source-preserving save and keyboard access.
- Contract exporter/generated types, focused tests, README and memory docs: align with actual implementation.

## Implementation steps
1. Add streaming hooks and report progress; verify output arrives before process exit and masking across read boundaries.
2. Add Runs/current-run UI, checked multi-step example; test navigation, returning to output and compact layouts.
3. Upgrade authoring editor with local bundled workers, syntax/completion/diagnostics and maximize; test save, Escape, keyboard access and exact source.
4. Run sequential checks and package the authorized main/tag release.

## Verification
Formatting, warning-denied Clippy, workspace Rust tests, generated contract freshness, strict frontend build and single-worker UI journeys. Focused native local run tests use disposable workspaces. Remote acceptance is read-only on authorized test nodes.

## Risks / edge cases
Retries and phases must not count as extra tasks. Handler work is separate from primary-step counts. Dynamic Ansible includes have no reliable fixed denominator; show completed phases rather than fabricated percentages. Output is bounded, and streamed previews are in-memory until phase completion. Interrupted SSH outcomes remain unknown. Bundled editor workers must work offline under CSP; no source execution for diagnostics.

## Out of scope
Continuing after app exit, remote daemon installation, automatic replay, a new YAML dialect, downloading interpreters/models or production changes.
