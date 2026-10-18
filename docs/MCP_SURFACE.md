# Ghosh MCP surface v1

Status: normative architecture contract, implementation scheduled M7.
Ghosh is both server and client. Rust `rmcp` adapts the shared application facade;
server methods must not implement separate SSH/browser/task business logic.
The [official Rust SDK](https://docs.rs/rmcp/latest/rmcp/) provides both roles and
[stdio/Streamable HTTP transports](https://docs.rs/rmcp/latest/rmcp/transport/index.html).
Pin a tested SDK version during implementation, not a floating latest dependency.

## Transports and lifecycle

- Negotiate a supported protocol version at initialize and advertise only
  implemented tools/resources/prompts/logging/subscription capabilities.
- `ghosh mcp --workspace <path>` is a stdio shim attaching to the running app's
  per-user Unix socket. It never starts an independent second workspace writer.
  Missing app returns a clean startup error to stderr; stdout stays MCP-only.
- Local socket directory is mode 0700, socket mode 0600; validate owner/peer UID,
  workspace canonical path, instance identity and socket lifetime. Per-client
  contexts/subscriptions are distinct; do not broadcast another client's replies.
- HTTP endpoint is `/mcp`, default bind `127.0.0.1:7431`; support Streamable HTTP,
  not legacy standalone SSE. Bind failure is visible; do not silently select a
  publicly reachable address. Multiple workspaces need explicit unique ports.
- Every HTTP request uses a per-workspace Keychain bearer token over loopback.
  Validate Host and any supplied Origin against exact allowlists; invalid Origin
  returns 403. No wildcard CORS. Native clients without Origin still require auth.
  Rotate tokens with session invalidation; never show tokens in logs/URLs.
- Nonloopback binds are refused. Future allow_remote=true is reserved, requires
  explicit secure deployment approval and authentication/TLS design; unsupported
  initially even if a user inserts that field manually.
- Follow [MCP transport requirements](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports)
  for framing, headers, sessions and explicit cancellation. The linked version is
  a reference, not a promise of a hardcoded sole protocol version. Determine the
  supported version set from the pinned SDK and conformance at M7.
- Long tools/call requests produce progress notifications when a token is supplied,
  structured logging when negotiated, and one final result. Disconnect is not
  cancellation. A cancel notification propagates through the operation token.

## Tools

All requests/results have JSON Schemas generated from core DTOs. Entity IDs are
validated IDs, selectors are normalized before permission and approval decisions.
Optional operation options include deadline and bounded output limits.

| Tool | Input / result | Mutates | Approval in write mode |
| --- | --- | --- | --- |
| workspace.info | {} → workspace ID/name/schema/revision/capabilities | no | no |
| node.list | selector? → redacted node summaries | no | no |
| node.facts | node, fresh=false → timestamped cached/probed facts | no | no; vetted probes only |
| node.exec | node, command, timeout_s? → exit/stdout/stderr/truncation | yes | required |
| node.run | playbook, targets?, check=true → RunReport | unless check | required unless safe check |
| tunnel.list | {} → route summaries | no | no |
| tunnel.status | tunnel → up/down/starting + safe health | no | no |
| tunnel.start | tunnel → status | yes | no tool approval; still scoped policy |
| tunnel.stop | tunnel → affected leases/status | yes | no tool approval; still scoped policy |
| profile.list | {} → profile summaries/capabilities | no | no |
| profile.open_tab | profile, url? → tab_id | yes | required |
| page.extract | tab_id, selector, fields?, limit? → rows | no | no; scoped read permission |
| page.screenshot | tab_id → bounded safe artifact content/reference | no | no; scoped read permission |
| page.eval | tab_id, js → bounded JSON value | yes | required |
| page.click | tab_id, selector(s), options? → success | yes | required |
| page.type | tab_id, selector(s), text OR value_ref → success | yes | required |
| page.run | profile, recording, vars? → per-step report | yes | required |
| cert.list | {} → domains/expiry summaries | no | no |
| cert.status | cert → expiry/sync/verification status | no | no |
| cert.issue | cert, force? → notAfter/fingerprint | yes | required |
| cert.sync | cert, targets? → per-node results | yes | required |
| recipe.plan | recipe, node → missing/drift/diff | no | no; vetted probes only |
| recipe.apply | recipe, node, parallel_downloads? → task results | yes | required |
| script.run | script, input, runtime?, targets? → run report | yes | required + script grant |
| term.run | command, cwd?, timeout_s? → exit/stdout/stderr | yes | required |

No secret read/write, arbitrary workspace file dump, raw socket or arbitrary HTTP
forwarding tool is exposed. Browser open/operation tools are withheld until the
native backend exists. Unsupported tools are absent, not success-shaped stubs.
`node.run(check=true)` validates every check and template helper as approved read
code; arbitrary shell probes from an untrusted workspace cannot use this exemption.
Network navigation may have website effects; scope and approval still apply.

## Read-only and approval policy

Default read_only=true hides every mutating tool including tunnel.start/stop.
`node.run` may be listed with a check property constrained to const true; omitted
check defaults true; false is rejected even by a direct tools/call. Policies are
rechecked at dispatch and immediately before effects. listTools filtering alone
is insufficient. Changing read_only triggers tools/list_changed if negotiated.

The Approval column in Appendix B1 of [the build prompt](ghosh-prompt.md) is
the source of truth. At server startup, validate that explicit `require_approval`
covers every unconditional Approval=yes tool plus `node.run`. Refuse startup with
a diagnostic listing missing tools; do not silently start with a weaker policy.
An omitted list uses the complete built-in default. Extra entries may require
approval for additional tools. Validate this even when read_only=true.

`node.run` is conditional per invocation: skip tool approval only for `check=true`;
otherwise require it regardless of list membership. Safe-check execution
permissions still apply. Policy reload repeats validation before becoming active.
Read tool permission checks, workspace trust and script capabilities still apply.

Native approval sheet shows requester, exact public command/arguments, resolved
target IDs, route, expected changes/diff, and any unavailable diff. Secret values
are placeholders; entered page text may be sensitive and is not logged. Allow once,
Allow for this session, Deny. Session grants are scoped to client identity,
workspace/revision, tool and canonical arguments/plan hash; not an entire-tool
wildcard. Changes of script/recording contents, targets, route or diff invalidate
the approval. Human user resolves the sheet; agents cannot call an approve tool.

A denied request produces isError=true with code APPROVAL_DENIED and safe context;
no browser/navigation/remote/process business effects occur. A denial audit event
is allowed and required. Permission denials/timeouts are equally fail-closed.
Audit is append-only application behavior in state/audit.jsonl, not a claim of
cryptographic tamper-proof storage. Never store tokens, secret text or raw auth.

## Results, output and errors

Standard tool results use content and optional structuredContent validated against
the advertised outputSchema. Domain failures use isError=true plus safe error
code; malformed protocol arguments use JSON-RPC invalid params. Approval/timeout/
rate-limit codes are documented domain errors, not fabricated standard RPC codes.

Remote-node and web-derived results carry `_meta` application metadata:
`app.ghosh/trust="untrusted"`, source kind/ID, and extraction timestamp. Structured
content also has trust/source fields and human text includes an untrusted label,
because clients may drop metadata. This is advisory provenance, never a security
sandbox or an instruction channel. Local run logs include mixed-origin provenance.

Default output cap 256 KiB per response, with visible `[truncated]` marker and
`truncated=true`, original bytes if known, safe cursor/artifact handle if available.
Never expose an unchecked arbitrary artifact path/URI. Images use appropriate MCP
image content or authorized resources; cap pixels/bytes. Per-client token bucket
60 requests/minute, 8 inflight; per-tool deadline defaults 600 s for long jobs and
30 s for read probes. All values may be lowered by policy. Mutating node leases
span all entry points, not merely one MCP client.

## Resources

| URI | Content and behavior |
| --- | --- |
| ghosh://workspace/manifest | validated TOML, secrets/auth fields redacted |
| ghosh://nodes/{id} | node TOML with references and auth identifiers safely redacted |
| ghosh://profiles/{id} | profile TOML excluding sensitive init payloads |
| ghosh://tunnels/{id} | tunnel TOML, credential fields removed |
| ghosh://certs/{id} | certificate config/status, no private keys/tokens |
| ghosh://recipes/{id} | recipe TOML, auth fields redacted |
| ghosh://playbooks/{name} | approved workspace playbook, scoped read only |
| ghosh://recordings/{id} | recording JSON with sensitive input values masked |
| ghosh://runs/{run_id}/log | bounded paginated redacted log, subscribable |
| ghosh://api/ghosh.d.ts | generated scripting declarations |

Use resources/list for concrete resources and templates/list for parameterized
resources. Validate IDs and percent decoding; reject path traversal/symlink escape.
Subscriptions receive resources/updated notifications and clients re-read/paginate;
notifications do not contain unbounded full log payloads. Membership changes use
resources/list_changed. All reads enforce the same scopes as tools and refuse
unreviewed executable/sensitive content; references alone do not authorize access.

## Prompts

- write_playbook(goal, targets): ghosh.d.ts, approved target facts and task contracts.
- fix_failed_run(run_id): bounded redacted log and failing task snapshot.
- record_to_script(recording_id): approved masked steps and page API declarations.

Facts/logs/recording contents are explicitly delimited untrusted data inside
prompts. Prompt retrieval never executes or approves the suggested operation.

## Ghosh as client

`ghosh.mcp.connect({name, command?, args?, url?, headers?})` yields listTools,
callTool, readResource, close. Exactly one transport is selected. Predeclared
[[mcp.clients]] entries resolve headers_ref in Rust. Dynamic entries require first
use approval showing resolved executable/args, cwd/env and endpoint/header names.
Script capabilities restrict allowed server names, origins, tools and paths.
A first-use grant is bound to executable or config identity; a changed command
invalidates it. Process stdio never contaminates the host application protocol.

Supervise process groups, drain stderr, bound frames, time out/cancel calls, and
close on run/workspace end unless an explicit session lifetime is granted. Do not
forward Ghosh bearer tokens to other servers or follow credential-leaking redirects.
External tool descriptions/results cannot expand permissions. Resolve tool
metadata as untrusted information and recheck caller scope for every call.

## Conformance and negative tests (B3)

1. Inspector: initialize/version, tools/list, read-only tools/call, resources/read,
   templates, prompts, logging and subscription/listChanged capability behavior.
2. Start a permitted long operation with progressToken, observe progress, cancel,
   verify final lifecycle and cleanup; disconnect must not implicitly cancel.
3. read_only hides all mutators; raw direct calls and check=false cannot bypass it.
4. Deny approval; verify zero business effects across shared facade fakes and real
   fixtures, with only a redacted denial audit entry.
5. Secret canaries through success/failure/stderr/progress/resources/prompt/audit
   paths; none may appear. Sensitive screenshots require dedicated masking tests.
6. Invalid Origin, Host, token, session, peer UID, forged workspace ID, rate flood,
   oversize payload, URI traversal and expired grant fail without effects.
7. Two clients start writes on the same node; the core lease serializes them.
8. A malicious check command, recording, page or external server cannot approve
   itself, invoke privileged shell commands or alter the target set.
9. The supplied A1 `ghosh.toml` covers every B1 mandatory approval; remove each
   mandatory entry in turn and verify startup fails with the missing tool named.
   Test omitted-list defaults, additive entries and invalid policy reload.
10. In write mode, each of profile.open_tab, page.eval, page.click, page.type and
    page.run opens a native approval sheet before effects. Denying each leaves
    browser state unchanged, apart from the allowed redacted audit event.
11. A profile manifest with a non-UUID data_store is rejected during load. A valid
    UUID survives save/reopen/clear; attempted identity edits are rejected.
12. node.run check=true avoids the tool sheet only after safe-check permission
    validation; check=false requires approval on every ungranted call. A changed
    plan/hash invalidates previous grants. Direct calls cannot bypass either rule.
