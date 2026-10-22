# Implementation Plan: reliable runs, system-key setup and notebook scripts

Status: Complete; approved by user (“go”). Date: 2026-10-19.

## Goal
Make Check versus Apply unambiguous, connect existing nodes with the user's system
file keys, clean finished run history, and provide a notebook-style script editor
with per-step output and a compact monitor that leaves the SSH terminal usable.

## Existing behavior and diagnosis
- Actual `1.ghosh/playbooks/ms-script.yml` checks for `~/ghosh-checked-example`
  and writes `123` there only during Apply.
- Its two latest local reports ran in Check mode: state succeeded, check exit 10,
  task state drift. The write command was never invoked. No execution bug is proven.
- Both saved nodes (`debian.orb.local`, `node3.dabaga.cc`) have schema 1 and explicit
  agent auth. The app's file-key registry is empty with no default.
- System key paths exist in ~/.ssh and ~/.orbstack/ssh. These are separate identities;
  the OrbStack identity must not become a global key for unrelated servers.
- Last increment only changed new schema-2 drafts; it did not rebuild the desktop app
  or alter existing workspace files.
- Runs persist bounded results and execute off the UI thread. History has no deletion.
  A footer button links to Runs, but does not display output. The command popup's
  global busy flag currently includes active runs, requiring narrower action guards.

## Proposed approach
1. Check stays non-writing for simple scenarios. Show “Check complete — changes
   needed” for drift, the target path when present in output, and a direct Apply
   action. Label Check “Inspect only” and Apply “Make changes”; do not infer that a
   successful check performed the change. Retain existing report schema states.
2. Add a system-key setup/recovery action using existing reviewed key registration,
   workspace schema migration and node updates. Discover candidate paths/metadata
   on explicit setup, show fingerprints, suggest OrbStack's key for *.orb.local
   and user system keys for other nodes, and let the user confirm exact bindings.
   No private key contents in UI/logs; no arbitrary ssh_config command execution.
   Existing explicit agent choices do not silently fall back. Recovery makes it
   possible to convert the two affected nodes rather than only new drafts.
3. Add Remove run and Clear finished runs, scoped to the active workspace.
   Remove JSON snapshots and JSONL metadata; reject running runs in the backend,
   validate UUIDs and workspace ownership, and show the concrete deletion scope.
4. Add Notebook scripts alongside Shell/Python/Deno under Scripts. Persist a
   versioned `.steps.yml` document with stable cell IDs, title, runtime, code and
   timeout. Start with code cells using installed Shell/Python/Deno runtimes.
   Provide Run cell, Run all, add/reorder/remove cells, per-cell status/output,
   and optional explicit success checks. Default execution runs code directly;
   it does not require a check command that deliberately reports drift.
5. Reuse the current native run worker, credentials, output caps and Stop handling.
   Run only saved revision-bound cells; validate selected cell IDs natively.
   Cells use fresh processes, with filesystem changes persisting on the target;
   variables and current-directory changes do not carry between cells. Display
   this plainly. Re-running a cell executes its code again.
6. Show a small modeless output panel with target, current cell, output, progress,
   Stop, minimize and Open full run. Leave terminal keyboard input usable while
   it is visible. Persist the run off-panel; closing the panel does not stop it.
   Permit opening/using SSH PTYs during a run while retaining workspace-change
   guards and the existing one-automation-run limit. Keep Ghosh open during runs.

## File changes
Modify existing files:
- src/features/automation/RunDialog.tsx: explicit mode labels, cell selection and
  SSH setup recovery entry; target retention after failure.
- src/features/automation/RunsView.tsx: contextual Check outcome, Apply and cleanup.
- src/features/automation/useRuns.tsx: shared refresh and compact monitor state.
- src/features/automation/AutomationView.tsx, examples.ts: notebook catalog/editor
  entry, existing scenario compatibility and file-writing example output.
- src/features/nodes/NodeEditor.tsx, src/features/keys/KeysView.tsx: reviewed system
  key suggestions and reuse of migration/key/node binding flows.
- src/app/App.tsx, commands.ts, ipc.ts, src/styles.css: modeless monitor, precise
  command availability, typed notebook/cleanup/key-discovery bridge and styling.
- crates/ghosh-core/src/automation.rs: notebook schema, kind, validation and bounds.
- crates/ghosh-core/src/runs.rs: saved-cell selection, execution and optional checks;
  compatible optional cell metadata and cleanup request DTOs.
- crates/ghosh-core/src/workspace/automation.rs: `.steps.yml` discovery/IDs/storage.
- src-tauri/src/services.rs: read-only candidate key discovery on explicit setup.
- src-tauri/src/services/runs.rs: scoped history cleanup and notebook dispatch.
- src-tauri/src/commands.rs, lib.rs, build.rs, permissions/automation.toml,
  permissions/commands.toml: register and authorize native local UI commands.
- crates/ghosh-core/examples/export_contracts.rs: export new DTOs; regenerate
  src/generated/contracts.ts and applicable schemas, never hand-edit generated code.
- crates/ghosh-core/tests/automation.rs, tests/ui/shell.spec.ts: notebook persistence,
  native validation and browser interaction coverage; native run tests in services/runs.rs.
- docs/PROJECT_CONTEXT.md, ARCHITECTURE_INDEX.md, MODULE_OWNERSHIP.md,
  API_CONTRACTS.md, DECISIONS.md, TASK_LOG.md and README.md: implemented contracts,
  screenshots, checks and limitations.
Create:
- src/features/automation/NotebookEditor.tsx: cell authoring with existing SourceEditor.
- src/features/automation/RunMonitor.tsx: compact non-blocking live output panel.
No files scheduled for deletion.

## Implementation steps
1. Correct Check presentation and prove Check leaves a temporary file absent while
   Apply creates it, verifies content, and fails when verification fails.
2. Implement candidate system-key discovery and reviewed setup. Verify missing,
   encrypted, unsafe and changed keys; verify separate OrbStack/remote bindings.
3. Add backend cleanup with active-run race checks, then deletion controls and
   refresh. Test foreign workspace IDs, malformed IDs, active runs and JSONL cleanup.
4. Add notebook schema/persistence, then native all/subset execution and durable
   cell metadata. Test mixed runtimes using installed tools, failure/Stop semantics,
   source revision changes, ordering and output caps.
5. Build notebook cell editor and monitor. Narrow terminal command guards so a
   running automation and an independent SSH PTY can coexist; workspace replacement
   stays guarded. Test output while navigating, dismissal, keyboard focus and 200% zoom.
6. Update memory/docs, run checks, rebuild the native macOS app and refresh the
   local release archive/checksums using existing scripts. Record actual evidence.
   Validate local ms-script Check/Apply with a temporary test path, then read-only
   SSH/PTY and script acceptance on the two authorized nodes. Do not install software
   or overwrite the user's example output as part of acceptance.

## Verification
- scripts/check.sh: formatting, warning-denied Clippy, workspace Rust tests,
  generated contract freshness, strict production build and sequential UI suite.
- Native local temporary-file acceptance and private run deletion tests.
- Read-only per-node connection/PTY acceptance; confirm SSH stays usable during
  a long-running read-only notebook cell. Keep remote operations sequential.
- Native macOS rebuilt-app smoke: correct modes, cell output, background monitor,
  terminal coexistence and cleanup. Report skipped/unavailable checks honestly.

## Risks / edge cases
- OrbStack and remote hosts can require different keys. Missing references never
  trigger another identity without user selection.
- User workspace schema migration and exact key assignments require concrete review
  in the app; preserve backups and all unrelated configuration.
- Existing reports remain readable and Check drift is interpreted for old records.
- A stopped/lost SSH command may still run remotely; keep outcome-unknown semantics.
- History cleanup must not delete a worker's active report or another workspace.
- Fresh processes mean this is notebook-style authoring, without shared kernel state.

## Out of scope
Real Jupyter kernels/.ipynb execution, package/runtime installation, unattended
execution after app exit, multiple simultaneous automation runs, arbitrary
ssh_config ProxyCommand/Match exec execution, publishing or GitHub release creation.
