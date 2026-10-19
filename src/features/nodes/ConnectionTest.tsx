import { Select } from "../../components/Select";
import { useEffect, useState } from "react";
import { Dialog } from "../../components/Dialog";
import { api, errorMessage } from "../../app/ipc";
import type { NodeConnectionPreview, NodeConnectionResult, WorkspaceSnapshot, TerminalSession } from "../../generated/contracts";

export function ConnectionTest({ workspace, target, mode = "test", onTerminal, dismiss, busyChanged }: { workspace: WorkspaceSnapshot; target?: string; mode?: "test" | "terminal"; onTerminal?: (session: TerminalSession) => void; dismiss: () => void; busyChanged: (busy: boolean) => void }) {
  const [node, setNode] = useState(target || workspace.nodes[0]?.id || "");
  const [preview, setPreview] = useState<NodeConnectionPreview | null>(null);
  const [result, setResult] = useState<NodeConnectionResult | null>(null);
  const [allowUnknown, setAllowUnknown] = useState(false);
  const [credential, setCredential] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [revision] = useState(workspace.revision);
  const stale = workspace.revision !== revision || workspace.external_changed || !workspace.trusted;
  useEffect(() => { busyChanged(busy); return () => busyChanged(false); }, [busy, busyChanged]);
  const review = async () => {
    setBusy(true); setError(null); setResult(null); setPreview(null); setAllowUnknown(false); setCredential("");
    try { setPreview(await api.connectionPreview({ context: { workspace_id: workspace.id, revision }, node_id: node })); }
    catch (e) { setError(errorMessage(e)); } finally { setBusy(false); }
  };
  const authenticate = async () => {
    if (!preview || busy || stale || (preview.host_key.trust === "unknown" && !allowUnknown)) return;
    setBusy(true); setError(null);
    try {
      const connection = { ticket: preview.ticket, allow_unknown: allowUnknown, credential: credential || null };
      setCredential("");
      if (mode === "terminal") {
        const session = await api.terminalOpenNode({ connection, dimensions: { rows: 24, cols: 80 } });
        if (onTerminal) onTerminal(session); else await api.terminalClose(session.id);
        dismiss();
      } else setResult(await api.connectionTest(connection));
    }
    catch (e) { setError(errorMessage(e)); } finally { setCredential(""); setPreview(null); setBusy(false); }
  };
  return <Dialog title="connection-test-title" dismiss={dismiss} busy={busy}><form onSubmit={e => { e.preventDefault(); if (!stale) void (preview ? authenticate() : review()); }}>
    <p className="eyebrow">SSH · DIRECT CONNECTION</p><h2 id="connection-test-title">{mode === "terminal" ? "Open node terminal" : "Test node connection"}</h2>
    {!preview && !result && <><label htmlFor="connection-node">Node</label><Select id="connection-node" data-dialog-focus value={node} disabled={busy} onChange={e => setNode(e.target.value)}>{workspace.nodes.map(item => <option key={item.id} value={item.id}>{item.id} · {item.user}@{item.host}:{item.port}</option>)}</Select><p className="field-help">Review connects to read the server's public fingerprint. The chosen authentication method is used only after your review. {mode === "terminal" ? "A real interactive shell will open on this node." : "No remote commands run."}</p></>}
    {preview && <><dl className="operation-details"><dt>Node</dt><dd>{preview.node_id}</dd><dt>Endpoint</dt><dd>{preview.user}@{preview.host}:{preview.port}</dd><dt>Host algorithm</dt><dd>{preview.host_key.algorithm}</dd><dt>Host fingerprint</dt><dd className="fingerprint">{preview.host_key.fingerprint}</dd><dt>Host trust</dt><dd>{preview.host_key.trust === "known" ? "Matches an existing known_hosts entry" : "Unknown — verify this fingerprint independently"}</dd><dt>Identity</dt><dd>{preview.identity.source} · {preview.identity.public?.algorithm}</dd><dt>Key fingerprint</dt><dd className="fingerprint">{preview.identity.public?.fingerprint}</dd></dl>
      {preview.host_key.trust === "unknown" && <label className="checkbox-label"><input type="checkbox" data-dialog-focus checked={allowUnknown} disabled={busy} onChange={e => setAllowUnknown(e.target.checked)} />I verified this server fingerprint. Allow it for this connection only.</label>}
      {preview.credential_prompt && <><label htmlFor="connection-credential">{preview.credential_prompt}</label><input id="connection-credential" data-dialog-focus type="password" autoComplete="off" maxLength={4096} value={credential} disabled={busy} onChange={event => setCredential(event.target.value)} /><p className="field-help">Used for this connection only. Never saved in Ghosh or Keychain.</p></>}
      <p className="field-help">Review expires after two minutes. The server key and chosen identity are checked again. Existing host trust files remain unchanged.</p></>}
    {result && <div role="status"><h3>Authenticated successfully</h3><p>{result.node_id} · {result.elapsed_ms} ms</p><p className="fingerprint">{result.host_key.fingerprint}</p><p className="field-help">Connection closed. No shell or remote command was started.</p></div>}
    {stale && <p role="alert" className="notice error">Resume workspace operations in Overview and reload changed files before connecting.</p>}
    {error && <p role="alert" className="form-error">{error}</p>}
    {busy && <p role="status">{preview ? "Verifying and authenticating…" : "Reading server fingerprint…"}</p>}
    <div className="modal-actions"><button type="button" data-dialog-focus={!!result || preview?.host_key.trust === "known" || undefined} disabled={busy} onClick={dismiss}>{result ? "Done" : "Cancel"}</button>{!result && <button className="primary" disabled={busy || stale || !node || (preview?.host_key.trust === "unknown" && !allowUnknown)}>{preview ? mode === "terminal" ? "Open SSH terminal" : "Authenticate" : "Review server"}</button>}</div>
  </form></Dialog>;
}
