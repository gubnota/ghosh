# Ghosh workflow status

Updated 2026-10-19. This is implementation status, not a release promise.
The approved operational plan remains in progress. Work is sequential.

| Workflow | Working now | Still missing |
| --- | --- | --- |
| Workspace lifecycle | New/Open/Close/restore-last, searchable recents, clear history, default approval | Schedule ownership when closing a workspace |
| Keyboard and layout | Physical RU/EN shortcuts, clickable Cmd+K, Cmd+W, Cmd+1–9, Cmd+[/], themed selectors and responsive containment | Acceptance for each new feature as it lands |
| SSH nodes and keys | Node CRUD, system/custom file keys, automatic safe system-file defaults, explicit key defaults, agent, encrypted key/passphrase, password implementation | Live password acceptance, agent identity picker, jumps, SSH-config import, SFTP |
| SSH connection | Test/Terminal connect immediately; only required credentials prompt; changed known keys rejected | Persistent enrollment of newly observed host keys, route-loss recovery |
| Terminal emulator | Real local and managed SSH PTYs, xterm, resize/split/search, bounded replay, cleanup | Richer session restoration/reconnect UI |
| Playbooks | Multistep-only authoring, full-window editor and steps, drag ordering, one-action Save, import/export/drop and restore-missing editable examples; live output, independent SSH shells, Stop and history cleanup/export | Remote agent/MCP execution integration, richer typed adapters |
| Legacy automation | Existing source files and reports remain intact; Scripts navigation is replaced by Monitor | Explicit migration tooling |
| Existing server-first-run | Real pinned upstream Runner; linked inventory/group_vars/roles/includes/handlers verified in a fixture | Import/preview all real entrypoints; full cross-host batch semantics; production certificate workflow acceptance |
| Certificates | Target workflow documented | Normal Certbot/Cloudflare renewal, selective certificate distribution, Nginx restart/verification |
| Timers | Approved design | Durable enabled schedules, recent-workspace catch-up and run reconciliation |
| Health and Telegram | Integration planning approved | Health checks; Telegram integration design and subsequent implementation |
| Monitoring | Independent local/SSH connection, editable journal/file log commands, bounded live output, CPU/RAM/disk metrics, visible-only probes and Stop/disconnect | Server-dependent remote process termination; broader target acceptance |
| ComfyUI / RunPod | Recipe wizard, existing node/path/Python, pinned plugins, checksum model downloads, GPU/disk/path checks and reviewable Ansible YAML | Actual GPU-node acceptance, exact model URLs/hashes/compatibility, Pod lifecycle and template-specific restart |
| Browsers | Reference UI and architecture documented | Browsers session tab, isolated profiles/agents/proxies/VLESS, actions, scripts and result handling (including Krea) |
| Recordings | Planned | Capture/edit/replay and result persistence |
| 3D scenes | Requested scope recorded | Objects/rig/animation/camera controls and 9:16, 16:9, 1:1 video exports |
| MCP / CLI | Core authorization infrastructure | Running adapters and integration acceptance |
| Releases | Tag-only macOS arm64 CI and native packaging scripts; v0.1.5 native notebook/file/output/concurrent-terminal acceptance passed; Linux CI job preserved as comments per user | Linux CI checks failed on v0.1.2; repair/re-enable later, signing/notarization |

No production certificate issuance, deployment, database migration, remote plugin
installation or model download has been performed. Native SSH acceptance uses the
authorized Debian OrbStack and node3 targets; disposable files are used for smoke checks.

Multistep Playbooks are the default Playbook format: full-window editor/steps, stable-ID
drag ordering, offline keyword/construction completion, syntax checks, inert file/drop import,
exact-source export and success/failure/timeout examples. Step 0 disables timeout (legacy -1 remains accepted)
while retaining Stop. Runs contains execution/history only. Cmd+W closes the selected Terminal session; Cmd+Shift+brackets cycles sessions, and holding Cmd+Option shows shortcut help. Legacy .steps.yml sources stay editable in their original folder.
