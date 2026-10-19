import { Select } from "../../components/Select";
import { useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../../app/ipc";
import { Dialog } from "../../components/Dialog";
import type { IdentityPreview, KeysSnapshot, Node, NodeChangePreview, NodeMutationRequest, WorkspaceSnapshot } from "../../generated/contracts";

export type NodeAction = "create" | "edit" | "import" | "remove";
type Props = { workspace: WorkspaceSnapshot; action: NodeAction; target?: string; dismiss: () => void; update: (workspace: WorkspaceSnapshot) => void; busyChanged: (busy: boolean) => void };
const emptyNode = (): Node => ({ schema: 1, id: "", host: "", port: 22, user: "", auth: { method: "agent", key_ref: null, key_id: null, password_ref: null, agent_socket: null }, labels: [], tunnel: "direct", jump: null });
const fields = (node: Node | null) => node ? [node.id, `${node.host}:${node.port}`, node.user, node.auth.method === "key" ? `key: ${node.auth.key_id || node.auth.key_ref || "inherited"}` : node.auth.method, node.labels.join(", ") || "—", node.tunnel, node.jump || "—"] : Array<string>(7).fill("—");
export function NodeEditor({ workspace, action, target, dismiss, update, busyChanged }: Props) {
  const context = useRef({ workspace_id: workspace.id, revision: workspace.revision }).current;
  const [nodes, setNodes] = useState<Node[] | null>(action === "edit" ? null : []);
  const [node, setNode] = useState<Node>(emptyNode);
  const [chosen, setChosen] = useState(target || "");
  const [keys, setKeys] = useState<KeysSnapshot | null>(null);
  const [workspaceDefault, setWorkspaceDefault] = useState<string | null>(null);
  const [source, setSource] = useState("");
  const [replace, setReplace] = useState(false);
  const [changed, setChanged] = useState(false);
  const [discard, setDiscard] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [review, setReview] = useState<{ mutation: NodeMutationRequest; preview: NodeChangePreview; identity: IdentityPreview | null } | null>(null);
  const inFlight = useRef(false);
  const stale = workspace.revision !== context.revision || workspace.external_changed || workspace.read_only;
  const run = async (task: () => Promise<void>) => {
    if (inFlight.current) return;
    inFlight.current = true; setBusy(true); busyChanged(true); setError("");
    try { await task(); } catch (e) { setError(errorMessage(e)); }
    finally { inFlight.current = false; setBusy(false); busyChanged(false); }
  };
  useEffect(() => {
    if (action !== "edit") return;
    let alive = true;
    void api.nodes(context).then(values => {
      if (!alive) return;
      setNodes(values);
      const selected = values.find(value => value.id === target);
      if (selected) setNode(selected);
    }).catch(e => { if (alive) setError(errorMessage(e)); });
    return () => { alive = false; };
  }, [action, context, target]);
  useEffect(() => {
    if (workspace.schema < 2 || !["create", "edit"].includes(action)) return;
    let alive = true;
    void (async () => {
      try { const values = await api.keys(); const id = await api.workspaceKey(context); if (alive) { setKeys(values); setWorkspaceDefault(id); } }
      catch (e) { if (alive) setError(errorMessage(e)); }
    })();
    return () => { alive = false; };
  }, [action, context, workspace.schema]);
  const close = () => { if (!busy) { if (changed) setDiscard(true); else dismiss(); } };
  const patch = (value: Partial<Node>) => { setNode(previous => ({ ...previous, ...value })); setChanged(true); };
  const preview = () => { void run(async () => {
    const change = action === "import" ? { kind: "import" as const, source, replace } : action === "remove" ? { kind: "remove" as const, id: target! } : { kind: action === "edit" ? "update" as const : "create" as const, node };
    const mutation = { context, change };
    const identity = (action === "create" || action === "edit") && node.auth.method === "key" ? await api.identity({ context, node }) : null;
    setReview({ mutation, preview: await api.previewNode(mutation), identity });
  }); };
  const title = action === "create" ? "Add node" : action === "edit" ? "Edit node" : action === "import" ? "Import node" : "Remove node";
  if (discard) return <Dialog title="node-discard" dismiss={() => setDiscard(false)} alert><h2 id="node-discard">Discard this node draft?</h2><p>The saved inventory will stay as it is.</p><div className="modal-actions"><button autoFocus data-dialog-focus onClick={() => setDiscard(false)}>Keep editing</button><button onClick={dismiss}>Discard draft</button></div></Dialog>;
  if (review) {
    const before = fields(review.preview.before), after = fields(review.preview.after);
    return <Dialog key="review" title="node-review" dismiss={() => setReview(null)} busy={busy} className="node-dialog" alert><p className="eyebrow">INVENTORY CHANGE</p><h2 id="node-review">Review node change</h2><p>{review.preview.after ? `Save ${review.preview.after.id} in this workspace.` : `Remove ${review.preview.before?.id} from this workspace.`} Existing connection reviews will expire.</p><div className="table-scroll"><table><thead><tr><th>Field</th><th>Saved</th><th>Proposed</th></tr></thead><tbody>{["Node", "Address", "User", "Authentication", "Labels", "Route", "Jump node"].map((label, index) => <tr key={label}><th>{label}</th><td>{before[index]}</td><td>{after[index]}</td></tr>)}</tbody></table></div>
      {review.identity && <p className="notice">Identity: {review.identity.source} · {review.identity.public?.fingerprint || "No usable file key selected"} · {review.identity.status || "requires explicit file assignment"}</p>}
      {stale && <p className="notice error" role="alert">The workspace changed. Close this draft and reload the workspace before reviewing again.</p>}{error && <p role="alert" className="form-error">{error}</p>}
      <div className="modal-actions"><button autoFocus data-dialog-focus disabled={busy} onClick={() => setReview(null)}>Back to draft</button><button className="primary" disabled={busy || stale} onClick={() => { void run(async () => { update(await api.commitNode({ mutation: review.mutation, digest: review.preview.digest })); dismiss(); }); }}>{busy ? "Saving…" : action === "remove" ? "Remove node" : "Save node"}</button></div></Dialog>;
  }
  return <Dialog key="draft" title="node-editor" dismiss={close} busy={busy} className="node-dialog"><p className="eyebrow">LOCAL INVENTORY</p><h2 id="node-editor">{title}</h2><p>{action === "import" ? "Paste one node definition in TOML. The full inventory is validated before saving." : action === "remove" ? `Review removal of ${target}. Nodes referenced by another node cannot be removed.` : "Define the connection. Saving runs nothing."}</p>
    <form onSubmit={event => { event.preventDefault(); if (!stale) preview(); }} onKeyDown={event => { if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "s") { event.preventDefault(); event.currentTarget.requestSubmit(); } }}>
      {action === "edit" && <><label htmlFor="node-choice">Node to edit</label><Select id="node-choice" autoFocus data-dialog-focus value={chosen} disabled={busy || !nodes || changed} onChange={event => { const next = nodes?.find(value => value.id === event.target.value); setChosen(event.target.value); if (next) setNode(next); setChanged(false); }}><option value="">{nodes ? "Choose a node" : "Loading nodes…"}</option>{nodes?.map(value => <option key={value.id}>{value.id}</option>)}</Select></>}
      {action === "import" ? <><label htmlFor="node-source">Node TOML</label><textarea id="node-source" autoFocus data-dialog-focus rows={14} maxLength={1024 * 1024} spellCheck={false} value={source} disabled={busy} onChange={event => { setSource(event.target.value); setChanged(true); }} required /><label className="checkbox-label"><input type="checkbox" checked={replace} disabled={busy} onChange={event => { setReplace(event.target.checked); setChanged(true); }} />Replace a node with the same ID</label><p className="field-help">Private keys and passwords belong outside workspace configuration. Imports preserve comments and extra fields.</p></> : action === "remove" ? <p className="field-help">A local recovery copy of the definition is retained. This does not remove anything from the remote machine.</p> : (action === "create" || chosen) && <fieldset disabled={busy} className="node-fields"><div className="form-grid"><div><label htmlFor="node-id">Node ID</label><input id="node-id" autoFocus={action === "create"} data-dialog-focus={action === "create" || undefined} pattern="[a-zA-Z0-9_-]+" maxLength={64} value={node.id} disabled={action === "edit"} required onChange={event => patch({ id: event.target.value })} /></div><div><label htmlFor="node-host">Host</label><input id="node-host" maxLength={253} value={node.host} required onChange={event => patch({ host: event.target.value })} /></div><div><label htmlFor="node-user">SSH user</label><input id="node-user" maxLength={64} value={node.user} required onChange={event => patch({ user: event.target.value })} /></div><div><label htmlFor="node-port">Port</label><input id="node-port" type="number" min={1} max={65535} value={node.port} required onChange={event => patch({ port: Number(event.target.value) })} /></div></div><label htmlFor="node-labels">Labels (comma separated)</label><input id="node-labels" value={node.labels.join(", ")} onChange={event => patch({ labels: event.target.value.split(",").map(value => value.trim()) })} onBlur={() => setNode(previous => ({ ...previous, labels: previous.labels.filter(Boolean) }))} /><div className="form-grid"><div><label htmlFor="node-route">Route ID</label><input id="node-route" value={node.tunnel} required onChange={event => patch({ tunnel: event.target.value })} /></div><div><label htmlFor="node-jump">Jump node ID (optional)</label><input id="node-jump" value={node.jump || ""} onChange={event => patch({ jump: event.target.value || null })} /></div></div><label htmlFor="node-auth">Authentication</label><Select id="node-auth" value={node.auth.method === "key" && node.schema === 1 ? "legacy" : node.auth.method} onChange={event => { const method = event.target.value as "agent" | "key" | "password"; patch({ schema: workspace.schema >= 2 ? 2 : 1, auth: {method, key_ref: null, key_id: null, password_ref: null, agent_socket: null} }); }}><option value="agent">SSH agent</option><option value="key" disabled={workspace.schema < 2 || !keys}>File key</option><option value="password" disabled={workspace.schema < 2}>Password (ask when connecting)</option>{node.auth.method === "key" && node.schema === 1 && <option value="legacy">Legacy key reference (requires migration)</option>}</Select>
        {node.auth.method === "agent" && <><label htmlFor="node-agent-socket">SSH agent socket (optional)</label><input id="node-agent-socket" placeholder="Use SSH_AUTH_SOCK" value={node.auth.agent_socket || ""} onChange={event => patch({ auth: { ...node.auth, agent_socket: event.target.value || null } })} /><p className="field-help">The first supported public identity is shown for review. Only that identity is tried. Choose a file key if you need a different identity.</p></>}
        {node.auth.method === "password" && <p className="field-help">The password is requested for each connection and never saved. Legacy password references need conversion to this mode.</p>}
        {node.auth.method === "key" && node.schema >= 2 && <><label htmlFor="node-key">SSH identity</label><Select id="node-key" disabled={!keys} value={node.auth.key_id || ""} onChange={event => patch({auth: {...node.auth, key_id: event.target.value || null}})}><option value="">Inherit workspace / global default</option>{node.auth.key_id && !keys?.keys.some(key => key.id === node.auth.key_id) && <option value={node.auth.key_id}>Missing: {node.auth.key_id}</option>}{keys?.keys.map(key => <option key={key.id} value={key.id} disabled={key.status !== "available" && key.id !== node.auth.key_id}>{key.label} · {key.status}</option>)}</Select><p className="fingerprint">{(() => { const id = node.auth.key_id || workspaceDefault || keys?.default_key; const key = keys?.keys.find(key => key.id === id); return `${node.auth.key_id ? "Node" : workspaceDefault ? "Workspace" : "Global"}: ${key ? `${key.public.fingerprint} · ${key.status}` : "No usable identity; choose a key"}`; })()}</p></>}
        <p className="field-help">{workspace.schema < 2 ? "Enable file identities in SSH Keys → Workspace identity. Migration requires review and preserves legacy references." : "Node selection overrides the workspace default, then the global default. Missing assignments never fall back silently."}</p></fieldset>}
      {stale && <p className="notice error" role="alert">The workspace changed. Close this draft and reload before editing.</p>}{error && <p role="alert" className="form-error">{error}</p>}<div className="modal-actions"><button type="button" autoFocus={action === "remove"} data-dialog-focus={action === "remove" || undefined} disabled={busy} onClick={close}>Cancel</button><button className="primary" disabled={busy || stale || (action === "edit" && (!nodes || !chosen)) || (action === "import" && !source.trim())}>{busy ? "Validating…" : "Review change"}</button></div>
    </form></Dialog>;
}
