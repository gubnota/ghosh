# Implementation Plan: Playbooks, monitoring and keyboard navigation

Status: Approved and implemented; verification recorded in TASK_LOG.md.

## Goal
Implement the user's five requested workflow changes: execution-only Runs,
multistep-only Playbooks with recoverable editable examples, a real Monitor tab,
easier terminal session navigation/closing, and a Command+Option shortcut overlay.

## Existing behavior
- Scripts remains a destination and command group; Playbooks still offers simple
  scenarios, Ansible and multistep documents through one editor.
- Runs lists active/past reports but also launches Apply from completed checks.
- Three multistep examples have stable IDs but currently open as template drafts.
- Terminal has independent native PTYs, an eight-session bound, session selection
  and a separate Close session control; app bracket shortcuts cycle app tabs.
- Native SSH already owns host verification, system-key resolution and credentials.
  Monitoring is not implemented. No usable Graphify graph exists; use local maps.

## Proposed approach
- Follow-up: use `0` as the displayed/documented no-timeout value for step code
  and success checks; Stop remains effective. Retain `-1` as a legacy unlimited
  alias so previously saved/imported Playbooks keep working. Positive limits remain
  1..3600 seconds; reject other negatives. Update examples, labels and tests.
- Align Runtime and Timeout labels and control baselines in the Create/Edit
  Playbook step form, including when labels wrap and in compact layouts.
- Remove the separate Validate prerequisite for saving a multistep Playbook.
  Save (including Cmd+S) performs the existing preview/digest/commit internally
  in one action. No step execution or clean lint result is required to save.
  Keep structural document checks, safe IDs, revision/conflict checks and atomic
  persistence; report structural errors inline without discarding the draft.
- Interpret Runs as active executions and previously executed history only. Start
  and rerun Playbooks from Playbooks. Keep output, Stop, export and history cleanup.
- Remove Scripts navigation/commands. Playbooks exposes only the existing multistep
  format, full-window editor, step expansion/reorder, checks, timeout and transfer.
  Preserve older saved source files and reports; do not delete or silently convert.
  Existing ComfyUI recipes remain separate; do not expose Ansible as a Playbook type.
- Add Restore example Playbooks. Save missing steps-success, steps-failure and
  steps-timeout through existing revision-bound validation/commit. Never overwrite
  an existing ID, including an edited or invalid document. Report restored/skipped
  IDs; sequential saves refresh revision after each commit. Examples become ordinary
  editable/deletable Playbooks; no startup write or execution.
- Replace Scripts with Monitor. User selects a local machine or configured node and
  explicitly connects. Show live logs, CPU usage, RAM and disk usage, sampled every
  five seconds while visible. Linux metrics use /proc and df without installations;
  unsupported metrics display Unavailable, never invented values. Include bounded
  journalctl recent/follow and file-tail presets plus an editable log command.
  Missing journalctl/access permissions produce actionable output, with no automatic
  sudo. Stop/disconnect and switching targets cancel/reap the previous work.
- Monitor uses a separate bounded native session and reuses authentication, trust,
  cancellation and output limits. No Playbook/run-history records for monitoring.
  Pause sampling when hidden; bound log buffers and avoid overlapping samples.
- Add a visible X to every terminal session tab. Cmd+W closes the selected session
  when Terminal is active; retain existing termination confirmation. Cmd+Shift+[
  and Cmd+Shift+] cycle open terminal sessions, wrap and focus their PTY. Unshifted
  Cmd+brackets continue cycling app tabs. Closing the last session leaves the
  Terminal screen ready to open another; app tab X retains guarded tab close.
- Holding Command+Option shows a shortcut overlay; releasing either modifier,
  Escape or window blur hides it. A command/menu action opens a persistent,
  keyboard-accessible version. List app, editor, terminal and dialog shortcuts with
  context/availability, including new combinations. Preserve physical RU/EN keys,
  composition, Option text entry and shell control sequences.

## File changes
- src/app/commands.ts, App.tsx, AppShell.tsx, keyboard.ts, tabs.ts
  - Modify routes, command availability, keyboard precedence and close dispatch.
- src/features/automation/AutomationView.tsx, examples.ts, RunsView.tsx
  - Modify multistep-only authoring, missing-example restoration and history-only UI.
- src/features/automation/NotebookEditor.tsx, crates/ghosh-core/src/automation.rs,
  runs.rs, crates/ghosh-core/tests/automation.rs, examples/playbooks/*.steps.yml
  - Accept/display 0 as unlimited, retain legacy -1 compatibility, align fields,
    update examples and verify save without an explicit Validate action.
- src/features/terminal/TerminalView.tsx
  - Modify per-session close affordance, shortcut handling and focus restoration.
- src/features/monitor/MonitorView.tsx
  - Create target selection, connection, metrics and bounded streaming log controls.
- src/components/ShortcutOverlay.tsx, src/features/settings/SettingsView.tsx
  - Create overlay and align documented shortcuts with shared shortcut definitions.
- crates/ghosh-core/src/monitor.rs, lib.rs
  - Create/register bounded monitor DTOs and metric parsing with explicit availability.
- src-tauri/src/services/monitor.rs, services.rs
  - Create native monitor lifecycle, local/SSH collectors and cancellation cleanup.
- src-tauri/src/services/ssh.rs, local_exec.rs, crates/ghosh-ssh/src/exec.rs
  - Reuse connection and execution lifecycle; make only required shared access changes.
- src-tauri/src/commands.rs, lib.rs, build.rs, permissions/commands.toml
  - Register monitor commands and restricted local-UI permissions.
- src/app/ipc.ts, crates/ghosh-core/examples/export_contracts.rs
  - Add typed monitor API; regenerate contracts and autogenerated permissions.
- src/styles.css
  - Modify terminal controls and add responsive Monitor/overlay styles.
- tests/ui/shell.spec.ts and focused Rust tests alongside owning modules
  - Update removed-format journeys; cover new behavior and native lifecycle/parsers.
- README.md, docs/{PROJECT_CONTEXT,ARCHITECTURE_INDEX,MODULE_OWNERSHIP,
  API_CONTRACTS,DECISIONS,TASK_LOG,FEATURE_STATUS}.md
  - Update implementation-aligned behavior, contracts and verification evidence.

## Implementation steps
1. Consolidate Playbooks/navigation and remove launch actions from Runs; preserve
   existing files and historical compatibility. Verify no Scripts destination or
   format selector and no run launcher in Runs. Implement one-action Save, aligned
   Runtime/Timeout controls and 0/unlimited execution with legacy -1 compatibility.
2. Add idempotent example restoration using current authoring API, including stale
   revision, partial failure and collision handling. Verify edited examples survive.
3. Implement native monitoring with bounded output, explicit connection/credential
   lifecycle, independent channels, real metrics and cancellable log follow.
   Verify parsers, missing tools, cancellation, disposal and resource limits.
4. Add Monitor UI and typed IPC, checking log follow while a Playbook and terminal
   run independently. No remote installs or changes to user key/workspace files.
5. Add session X/close shortcut, node-session switching and shared shortcut overlay.
   Verify focus, keyboard layouts, modifier release/blur and modal precedence.
6. Update docs, run sequential checks, rebuild macOS app/archive and verify checksum.
   Preserve any open desktop draft; do not restart/discard it for acceptance.

## Verification
- scripts/check.sh: formatting, Clippy, Rust tests, contract freshness, frontend
  production build and sequential browser workflows.
- Native monitor acceptance: local CPU/RAM/disk readings, recent logs, live follow
  cancellation and cleanup; opt-in read-only checks on previously authorized
  OrbStack Linux/node3 where access is available.
- Browser checks: editable/restorable examples, no overwrite on repeated restore,
  import/export/drop, Runs history/Stop/cleanup, monitor unavailable/error/stream
  states, simultaneous terminal/run/monitor, session X and keyboard shortcuts,
  overlay dismissal and small-window containment.
- Timeout checks: 0 and legacy -1 execute without deadlines for both step/check;
  Stop cancels either; positive timeout still expires; other negatives are rejected.
  Save works immediately without Validate or running steps, including Cmd+S;
  syntax diagnostics do not block it. Verify Runtime/Timeout control alignment.
- scripts/build-release.sh v0.1.5; archive checksum; git diff --check.
- Report native/GUI gaps explicitly; fixtures do not prove a native connection.

## Risks / edge cases
- Older documents remain on disk but no longer appear in multistep-only Playbooks;
  old reports stay readable. No destructive migration is part of this change.
- Monitoring tools/permissions vary by target. Keep errors and missing metrics clear.
- Credential prompts and monitor sessions must not reuse terminal input or leak
  secrets into output/history. Closing workspace cancels all monitor work.
- Global keyboard handling currently ignores Option combinations; overlay needs
  explicit precedence without intercepting ordinary Option typing or shell Ctrl.

## Out of scope
Remote installations, sudo escalation, durable monitoring after app exit, alerts,
metrics history/database, automatic legacy conversion, deleting legacy documents,
new Playbook formats, Git publication and replacing user credentials.
