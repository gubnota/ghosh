# Ghosh scripting API v1

Status: architecture contract; not executable bindings. User Appendix A9 is
preserved below. The refinements following the declaration are normative.

```ts
declare const ghosh: Ghosh;

interface Ghosh {
  workspace: {
    path: string;
    name: string;
    read(rel: string): Promise<string>;
    write(rel: string, data: string | Uint8Array): Promise<void>;
  };
  node(id: string): Node;
  nodes(selector?: string): Node[]; // "labels:gpu", "node:gpu1"
  run(playbook: string, opts?: RunOpts): Promise<RunReport>;
  profile(id: string): Profile;
  tunnel(id: string): Tunnel;
  cert(id: string): Cert;
  recipe(id: string): Recipe;
  term: Terminal;
  secret(ref: string): Promise<string>; // requires permission, never logged
  log: {
    info(...a: any[]): void;
    warn(...a: any[]): void;
    error(...a: any[]): void;
  };
  sleep(ms: number): Promise<void>;
  mcp: McpClient; // call external MCP servers (see Appendix B2)
  input: Record<string, any>; // run parameters
}

interface Node {
  id: string;
  facts(): Promise<Record<string, any>>;
  exec(
    cmd: string,
    o?: { timeout_s?: number; sudo?: boolean; env?: Record<string, string> },
  ): Promise<{ code: number; stdout: string; stderr: string }>;
  task: {
    file(a: {
      path: string;
      content?: string;
      src?: string;
      mode?: string;
      owner?: string;
    }): Promise<TaskResult>;
    shell(a: {
      cmd: string;
      check?: string;
      creates?: string;
      unless?: string;
    }): Promise<TaskResult>;
    apt(a: {
      name: string | string[];
      state: "present" | "absent" | "latest";
    }): Promise<TaskResult>;
    systemd(a: {
      name: string;
      state: "started" | "stopped" | "restarted" | "reloaded";
      enabled?: boolean;
    }): Promise<TaskResult>;
    git(a: {
      repo: string;
      dest: string;
      commit?: string;
    }): Promise<TaskResult>;
    download(a: {
      url: string;
      dest: string;
      sha256?: string;
      auth?: string;
    }): Promise<TaskResult>;
  };
  sftp: {
    put(local: string, remote: string): Promise<void>;
    get(remote: string, local: string): Promise<void>;
  };
}

interface TaskResult {
  status: "ok" | "changed" | "skipped" | "failed";
  diff?: string;
  message?: string;
}
interface RunOpts {
  check?: boolean;
  concurrency?: number;
  vars?: Record<string, any>;
}
interface RunReport {
  perNode: Record<string, TaskResult[]>;
  summary: { ok: number; changed: number; skipped: number; failed: number };
}

interface Profile {
  id: string;
  openTab(url?: string): Promise<Page>;
  tabs(): Promise<Page[]>;
  setUserAgent(ua: string): Promise<void>;
  setTunnel(id: string): Promise<void>;
  clearData(kinds?: ("cookies" | "storage" | "cache")[]): Promise<void>;
}

interface Page {
  goto(url: string): Promise<void>;
  click(sel: string | string[]): Promise<void>;
  type(sel: string | string[], text: string): Promise<void>;
  waitFor(
    sel: string | string[],
    o?: { timeout_ms?: number; state?: "visible" | "attached" },
  ): Promise<void>;
  eval<T = any>(js: string): Promise<T>;
  extract(sel: string, fields?: Record<string, string>): Promise<any[]>;
  screenshot(rel?: string): Promise<string>;
  saveData(
    rel: string,
    rows: any[],
    format?: "csv" | "json" | "sqlite",
  ): Promise<void>;
  replay(recordingId: string, vars?: Record<string, any>): Promise<void>;
  record: { start(): Promise<void>; stop(): Promise<string> }; // returns recording id
  close(): Promise<void>;
}

interface Tunnel {
  id: string;
  status(): Promise<"up" | "down" | "starting">;
  start(): Promise<void>;
  stop(): Promise<void>;
  localProxy(): Promise<string>;
}

interface Cert {
  issue(o?: {
    force?: boolean;
  }): Promise<{ notAfter: string; fingerprint: string }>;
  renewIfDue(): Promise<boolean>;
  sync(selector?: string): Promise<Record<string, TaskResult>>;
  verify(
    selector?: string,
  ): Promise<Record<string, { served: string; expected: string; ok: boolean }>>;
}

interface Recipe {
  plan(node: string): Promise<{ missing: string[]; drift: string[] }>;
  apply(
    node: string,
    o?: { parallel_downloads?: number },
  ): Promise<TaskResult[]>;
  verify(node: string): Promise<{ ok: boolean; problems: string[] }>;
}

interface Terminal {
  open(o?: { node?: string; cwd?: string }): Promise<TermSession>;
}
interface TermSession {
  send(text: string): Promise<void>;
  expect(
    pattern: string | RegExp,
    o?: { timeout_ms?: number },
  ): Promise<string>;
  close(): Promise<void>;
}

interface McpClient {
  connect(o: {
    name: string;
    command?: string;
    args?: string[];
    url?: string;
    headers?: Record<string, string>;
  }): Promise<McpSession>;
}
interface McpSession {
  listTools(): Promise<
    { name: string; description?: string; inputSchema: any }[]
  >;
  callTool(name: string, args: Record<string, any>): Promise<any>;
  readResource(uri: string): Promise<any>;
  close(): Promise<void>;
}
```

## Binding and runtime rules

JS and Node use the names above. Python uses equivalent async objects and
snake_case (`open_tab`, `set_user_agent`, `renew_if_due`, `list_tools`, `call_tool`,
`read_resource`, `save_data`, `wait_for`, `clear_data`, `local_proxy`). `perNode`
becomes `per_node`; options use snake_case consistently. Python regex is serialized
as source plus supported flags; JS RegExp and Python regex constructs outside
the shared RE2-style subset are rejected. Bytes cross stdio as explicit base64
objects, never ambiguous strings. Time values are UTC RFC3339 strings; absent
values are omitted, not arbitrary null coercions.

All APIs delegate to the permission-checked Rust application facade. Runtime
adapters never bypass it. Getters such as `node(id)` return opaque handles;
selection does not grant execution permission. Node selectors use node:<id> and
labels:<label>; multiple target selectors are a union with stable ID order.
Every API call inherits operation deadline, cancellation and caller identity.
`RunOpts` adds optional targets:string[] and timeout_ms:number. `RunReport` adds
run_id, check:boolean and lifecycle:running/completed/failed/cancelled/outcome_unknown;
existing fields stay unchanged. Failed mutating commands include partial effects.

`Page` adds readonly id:string, key(key, options?), scroll(x,y,selector?),
assert(assertion), and replayReport(recordingId, vars?) returning per-step status,
duration and redacted error. Existing replay() still resolves void or throws;
its report is retrievable from operation history. These additions cover recorded
key/scroll/assert steps absent from the original A9. `TermSession` adds resize(cols,
rows) and subscription through the host event bridge, not raw shell parsing.
`Recipe` adds export(node, options?) to satisfy recipe export. Generation produces
one ghosh.d.ts from core DTOs plus binding helpers, and a documented Python stub.

`workspace.read/write` operate within granted workspace-relative paths only.
`Page.saveData` uses the same guard. SQLite is local parameterized table insertion,
not arbitrary SQL. `sftp.get` cannot escape its granted local path. Download auth,
MCP headers and certificate secrets resolve inside Rust whenever possible.
`secret(ref)` is exceptional: explicit scoped permission, no logging or serializing
its return to UI/MCP; zeroization best effort, not an absolute memory guarantee.
`McpClient.connect` with headers is transient only; predeclared headers_ref is the
recommended path. Host redaction registers every resolved secret before use.

Errors are rejected promises/Python exceptions with code, message, retryable,
operation_id and safe details. Stable codes: INVALID_ARGUMENT, NOT_FOUND,
WORKSPACE_CONFLICT, UNSUPPORTED_SCHEMA, PERMISSION_DENIED, APPROVAL_DENIED,
HOST_KEY_UNKNOWN, HOST_KEY_MISMATCH, ROUTE_UNAVAILABLE, TIMEOUT, CANCELLED,
OUTPUT_LIMIT, REMOTE_FAILED, INTEGRITY_FAILED, OUTCOME_UNKNOWN, UNSUPPORTED.
Never expose raw credential-bearing error chains.

## Runtime selection and isolation

Use deno_core/V8 for JS with no Deno/Node ambient APIs or native module loading;
register only vetted host ops. Python and Node use supervised per-run child
process groups and JSON-RPC 2.0 newline-delimited messages. Both stdout and stderr
from user code route through bounded logging; stdout at the bridge layer is
protocol-only. Frames are limited to 1 MiB, operation output to 256 KiB by default;
large assets use scoped artifact handles, never unbounded JSON frames.

QuickJS is an explicit alternate engine if a measured V8 packaging/JIT spike
fails; it is not a silent automatic fallback. Ship a clearly selected runtime
and test its compatible JS subset. Python/Node runtimes are pinned, signed managed
sidecars; user-selected interpreters are separately identified as trusted local
code. V8 memory/time limits and cancellation are mandatory.

API permissions restrict Ghosh operations. Python/Node are trusted local programs
and may access the OS through their own libraries; a child process and JSON-RPC
are not an OS security sandbox. UI must say this before execution. Untrusted
scripts must stay disabled until an independently validated isolation strategy
exists. Embedded JS has less ambient authority, but is not a promise against
engine vulnerabilities. Script grants bind source hash and scope; editing a script
or broadening a referenced MCP server invalidates grants.
