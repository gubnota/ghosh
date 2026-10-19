import { Select } from "../../components/Select";
import { useEffect, useRef, useState } from "react";
import { Dialog } from "../../components/Dialog";
import { api, errorMessage } from "../../app/ipc";
import type { NodeConnectionPreview, NodeConnectionResult, WorkspaceSnapshot, TerminalSession } from "../../generated/contracts";

export function ConnectionTest({ workspace, target, mode = "test", onTerminal, dismiss, busyChanged }: { workspace: WorkspaceSnapshot; target?: string; mode?: "test" | "terminal"; onTerminal?: (session: TerminalSession) => void; dismiss: () => void; busyChanged: (busy: boolean) => void }) {
  const [node, setNode] = useState(target || workspace.nodes[0]?.id || "");
  const [preview, setPreview] = useState<NodeConnectionPreview | null>(null);
  const [result, setResult] = useState<NodeConnectionResult | null>(null);
  const [credential, setCredential] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [revision] = useState(workspace.revision);
  const credentialInput = useRef<HTMLInputElement>(null);
  const started = useRef(false);
  const inFlight = useRef(false);
  const stale = workspace.revision !== revision || workspace.external_changed || !workspace.trusted;
  useEffect(() => { busyChanged(busy); return () => busyChanged(false); }, [busy, busyChanged]);
  useEffect(() => { if (preview && !busy) credentialInput.current?.focus(); }, [preview, busy]);
  const authenticate = async (prepared: NodeConnectionPreview, secret: string) => {
    // The native single-use ticket still pins the exact server and identity.
    // The user requested direct SSH without an enrollment/approval dialog.
    const connection = { ticket: prepared.ticket, allow_unknown: true, credential: secret || null };
    setCredential(""); setPreview(null);
    if (mode === "terminal") {
      const session = await api.terminalOpenNode({ connection, dimensions: { rows: 24, cols: 80 } });
      if (onTerminal) onTerminal(session); else await api.terminalClose(session.id);
      dismiss();
    } else setResult(await api.connectionTest(connection));
  };
  const connect = async () => {
    if (inFlight.current || stale || !node) return;
    inFlight.current = true; setBusy(true); setError(null); setResult(null);
    try {
      if (preview) await authenticate(preview, credential);
      else {
        setCredential("");
        const prepared = await api.connectionPreview({ context: { workspace_id: workspace.id, revision }, node_id: node });
        if (prepared.credential_prompt) setPreview(prepared);
        else await authenticate(prepared, "");
      }
    } catch (e) { setError(errorMessage(e)); setPreview(null); }
    finally { setCredential(""); inFlight.current = false; setBusy(false); }
  };
  useEffect(() => {
    // StrictMode replays effects; one inventory action must open only one PTY.
    if (target && !started.current) { started.current = true; void connect(); }
  }, []);
  return <Dialog title="connection-test-title" dismiss={dismiss} busy={busy}><form onSubmit={e => { e.preventDefault(); void connect(); }}>
    <p className="eyebrow">SSH · DIRECT CONNECTION</p><h2 id="connection-test-title">{mode === "terminal" ? "Open node terminal" : "Test node connection"}</h2>
    {!preview && !result && <>{!target && <><label htmlFor="connection-node">Node</label><Select id="connection-node" data-dialog-focus value={node} disabled={busy} onChange={e => setNode(e.target.value)}>{workspace.nodes.map(item => <option key={item.id} value={item.id}>{item.id} · {item.user}@{item.host}:{item.port}</option>)}</Select></>}<p className="field-help">{target && `${target} · `}{mode === "terminal" ? "A real interactive shell will open on this node." : "Test SSH login without running a remote command."}</p></>}
    {preview && <><p>{preview.user}@{preview.host}:{preview.port}</p><label htmlFor="connection-credential">{preview.credential_prompt}</label><input ref={credentialInput} id="connection-credential" data-dialog-focus type="password" autoComplete="off" maxLength={4096} value={credential} disabled={busy} onChange={event => setCredential(event.target.value)} /><p className="field-help">Used for this connection only. Never saved in Ghosh or Keychain.</p></>}
    {result && <div role="status"><h3>Authenticated successfully</h3><p>{result.node_id} · {result.elapsed_ms} ms</p><p className="fingerprint">{result.host_key.fingerprint}</p><p className="field-help">Connection closed. No shell or remote command was started.</p></div>}
    {stale && <p role="alert" className="notice error">Resume workspace operations in Overview and reload changed files before connecting.</p>}
    {error && <p role="alert" className="form-error">{error}</p>}
    {busy && <p role="status">Connecting…</p>}
    <div className="modal-actions"><button type="button" data-dialog-focus={!!result || undefined} disabled={busy} onClick={dismiss}>{result ? "Done" : "Cancel"}</button>{!result && <button className="primary" disabled={busy || stale || !node}>{error ? "Retry connection" : "Connect"}</button>}</div>
  </form></Dialog>;
}
