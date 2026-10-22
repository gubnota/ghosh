# Implementation Plan: default multistep Playbooks and system SSH keys

Status: Approved by user (go). Date: 2026-10-19.

## Goal
Make multistep Playbooks the default authoring format, with whole-window maximize,
draggable steps and clear running-step progress. Use the current user's system SSH
file identities by default, including the existing legacy agent-default nodes,
without a separate system-key setup button.

## Existing behavior
- Notebook kind is stored as scripts/*.steps.yml and shown under Scripts. New
  Playbook opens the old scenario example. Only a single cell editor expands.
- Steps have stable IDs and move buttons; drag reordering is absent.
- completed_steps excludes the active step. The display says 0/N while step 1 runs.
- Node auth explicitly selects key/agent/password. Schema-1 nodes from the previous
  default still select agent even after the new-node form default changed.
- File identities currently require registration and schema-2 bindings. SystemKeySetup
  is an additional manual workflow; connection/automation/PTY share native SSH review.
- graphify-out contains only partial cache, no queryable graph; use local source
  mapping. Do not retry the previously rejected external source upload.

## Proposed approach
- Retain the internal notebook kind and saved schema for compatibility; label it
  Multistep Playbook throughout the UI, using steps rather than cells.
- Default New Playbook to this format. Store new .steps.yml in playbooks/;
  discover existing scripts/*.steps.yml as Playbooks and edit them in place.
  Preserve scenario, Ansible, and standalone Shell/Python/Deno formats.
- Add one maximize/restore control for the complete authoring/view surface, retaining
  all steps and save/run controls. Per-step editor resizing must not replace it.
- Drag from step handles; reorder by stable ID, preserve source/save semantics and
  keep keyboard move controls. Disable reorder during review/save and restore focus.
- Display Running step 1 of N separately from 0 of N completed. Use an animated
  activity indication for the active step, never invent elapsed-command percentages.
  Completed success alone reaches 100%; failures/stops keep accurate completed count.
- Add explicit System keys authentication, default for new nodes and legacy schema-1
  agent defaults with no custom socket. Preserve deliberately configured schema-2
  agents, custom agent sockets, explicit file keys and password choices.
- Resolve safe current-user conventional files at connection time, without registry
  imports or workspace migration: OrbStack identity for *.orb.local; otherwise
  ~/.ssh/id_ed25519, then ~/.ssh/id_rsa if Ed25519 is absent. An unsafe/changed
  chosen file fails clearly; authentication rejection does not silently try other keys.
- Pin the selected path/public fingerprint to the normal native connection review
  and revalidate before use. Passphrases retain existing per-connection handling.
  Apply the same identity to PTY, native automation and Ansible Runner.
- Remove the Use system file keys recovery button and its dedicated setup wizard;
  retain ordinary SSH Keys management and explicit custom identity choices.

## File changes
- src/features/automation/AutomationView.tsx, NotebookEditor.tsx, examples.ts:
  modify catalog/defaults/terminology, full-editor maximize and drag-step ordering.
- src/features/automation/SourceEditor.tsx, src/styles.css: modify whole-surface
  sizing, independent scrolling, drag handles and active-step presentation.
- src/features/automation/useRuns.tsx, RunsView.tsx, RunMonitor.tsx, RunDialog.tsx:
  modify running/completed labels, Playbook wording and remove setup recovery path.
- crates/ghosh-core/src/automation.rs, workspace/automation.rs: classify notebook as
  Playbook, recognize .steps.yml in both locations, preserve existing file paths.
- crates/ghosh-core/src/workspace/model.rs, workspace/identity.rs:
  add System authentication and explicit legacy-default compatibility policy.
- src-tauri/src/services.rs, services/ssh.rs, platform/ssh_keys.rs:
  resolve/inspect system identities, bind/recheck native tickets and use pinned key
  consistently for connection tests, PTYs and automation/Runner.
- src/features/nodes/NodeEditor.tsx, src/features/keys/KeysView.tsx:
  default/select System keys, show effective identity, remove setup wizard entry.
- src/features/keys/SystemKeySetup.tsx: delete once its callers are removed.
- src/generated/contracts.ts: regenerate from Rust, never hand-edit.
- crates/ghosh-core/tests/automation.rs, tests/identity.rs,
  src-tauri/src/services/ssh.rs native tests and tests/ui/shell.spec.ts:
  extend compatibility, system key selection/pinning, drag persistence, maximize,
  step-1 activity and completed/failure coverage.
- docs/PROJECT_CONTEXT.md, ARCHITECTURE_INDEX.md, MODULE_OWNERSHIP.md,
  API_CONTRACTS.md, DECISIONS.md, TASK_LOG.md, FEATURE_STATUS.md, README.md:
  update implemented behavior, compatibility policy and verification evidence.

## Implementation steps
1. Update core auth/format contracts and compatibility tests. Regenerate contracts.
2. Implement native System identity resolution and pinning; verify missing/unsafe/
   changed/encrypted files and explicitly selected authentication precedence.
3. Update Playbook catalogs/default creation; verify old sources remain readable
   and editable in place, including duplicate ID protections across both folders.
4. Implement complete-editor maximize/restore and drag ordering, retaining IDs,
   keyboard alternatives, dirty draft protection and saved execution order.
5. Update progress and terminology consistently across editor, dialog, Runs and
   compact monitor; remove obsolete system-key setup flow.
6. Run checks sequentially, rebuild app/archive and perform native acceptance.
   Update compact documentation with actual evidence and checksum verification.

## Verification
- scripts/check.sh sequentially: formatting, Clippy, Rust tests, generated contracts,
  strict production build and browser journeys.
- Browser: New Playbook defaults to multistep; no notebook-script wording; drag a
  step and save/reopen/run; maximize shows all steps/controls at narrow widths and
  200% zoom; step 1 has visible activity before any task completes.
- Native read-only connection/PTY/multistep acceptance on authorized OrbStack/node3
  using default system identities without registry setup or a running agent.
- Native temporary local Playbook, output/terminal coexistence and whole-editor
  maximize; rebuild macOS arm64 archive and verify SHA256SUMS.

## Risks / edge cases
- Legacy schema-1 agent/no-socket is indistinguishable from an intentional old
  agent choice: document compatibility change and expose explicit agent selection.
- Do not reinterpret custom agent sockets or registered/password identity choices.
- Two folders may contain colliding IDs: reject ambiguity, never overwrite/move
  user files automatically. Existing history keeps its internal kind and IDs.
- A command has no reliable partial percentage; show activity and completed work
  as different facts. Multiple target runs retain total planned-step counts.

## Out of scope
Full Jupyter kernels, arbitrary ssh_config execution, system SSH configuration
edits, installing keys/runtimes remotely, automatic private-key copying, multiple
simultaneous automation runs, execution after app exit and publishing a release.

## Authorized follow-ups
User steering within this increment: whole editor opens maximized, complete individual
steps expand independently, code highlighting/lint/keyword and construction completion,
source export/import and file drop, successful/failing/timeout examples, and -1 means
no timeout. Imports remain inert drafts; runtimes remain installed by the operator.
