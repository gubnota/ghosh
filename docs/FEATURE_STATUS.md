# Ghosh workflow status

Updated 2026-10-19. This is implementation status, not a release promise.
The approved operational plan remains in progress. Work is sequential.

| Workflow | Working now | Still missing |
| --- | --- | --- |
| Workspace lifecycle | New/Open/Close/restore-last, searchable recents, clear history, default approval | Schedule ownership when closing a workspace |
| Keyboard and layout | Physical RU/EN shortcuts, clickable Cmd+K, Cmd+W, Cmd+1–9, Cmd+[/], themed selectors and responsive containment | Acceptance for each new feature as it lands |
| SSH nodes and keys | Node CRUD, system/custom file keys, defaults, agent, encrypted key/passphrase, password implementation | Live password acceptance, agent identity picker, jumps, SSH-config import, SFTP |
| SSH connection | Test/Terminal connect immediately; only required credentials prompt; changed known keys rejected | Persistent enrollment of newly observed host keys, route-loss recovery |
| Terminal emulator | Real local and managed SSH PTYs, xterm, resize/split/search, bounded replay, cleanup | Richer session restoration/reconnect UI |
| Playbooks | Checked multi-step scripts and standard Ansible; offline maximizable editor, live logs/progress, navigation during runs, Check/Apply/Stop and report export | Remote agent/MCP execution integration, richer typed adapters |
| Scripts | Offline Monaco syntax/completion/diagnostics, explicit ID errors, Shell/Python/Deno examples and local/SSH execution with exit status, runtime checks, JSON inputs and report export | Result bindings, artifacts, secret inputs and configurable Deno permissions |
| Existing server-first-run | Real pinned upstream Runner; linked inventory/group_vars/roles/includes/handlers verified in a fixture | Import/preview all real entrypoints; full cross-host batch semantics; production certificate workflow acceptance |
| Certificates | Target workflow documented | Normal Certbot/Cloudflare renewal, selective certificate distribution, Nginx restart/verification |
| Timers | Approved design | Durable enabled schedules, recent-workspace catch-up and run reconciliation |
| Health and Telegram | Integration planning approved | Health checks; Telegram integration design and subsequent implementation |
| Monitoring | Remote interactive shell available | On-demand log streams and CPU/memory samples/charts with stop/cancel |
| ComfyUI / RunPod | Recipe wizard, existing node/path/Python, pinned plugins, checksum model downloads, GPU/disk/path checks and reviewable Ansible YAML | Actual GPU-node acceptance, exact model URLs/hashes/compatibility, Pod lifecycle and template-specific restart |
| Browsers | Reference UI and architecture documented | Browsers session tab, isolated profiles/agents/proxies/VLESS, actions, scripts and result handling (including Krea) |
| Recordings | Planned | Capture/edit/replay and result persistence |
| 3D scenes | Requested scope recorded | Objects/rig/animation/camera controls and 9:16, 16:9, 1:1 video exports |
| MCP / CLI | Core authorization infrastructure | Running adapters and integration acceptance |
| Releases | Tag-only macOS arm64 CI and native packaging scripts; Linux CI job preserved as comments per user | Linux CI checks failed on v0.1.2; repair/re-enable later, desktop acceptance, signing/notarization |

No production certificate issuance, deployment, database migration, remote plugin
installation or model download has been performed. Native SSH acceptance uses the
authorized Debian OrbStack and node3 targets; disposable files are used for smoke checks.
