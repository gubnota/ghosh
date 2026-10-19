import { Icon } from "../../components/Icon";
import { useEffect, useState } from "react";
import type { WorkspaceSnapshot } from "../../generated/contracts";
import { api } from "../../app/ipc";

type Props = { workspace: WorkspaceSnapshot; busy: boolean; run: (action: () => Promise<void>) => void; update: (s: WorkspaceSnapshot) => void; dirtyChanged: (dirty: boolean) => void; confirm: (title: string, action: () => void) => void };
export function WorkspaceView({ workspace: w, busy, run, update, dirtyChanged, confirm }: Props) {
  const [name, setName] = useState(w.name);
  const dirty = name !== w.name;
  useEffect(() => { setName(w.name); }, [w.id, w.name]);
  useEffect(() => { dirtyChanged(dirty); return () => dirtyChanged(false); }, [dirty, dirtyChanged]);
  const revision = { workspace_id: w.id, revision: w.revision };
  const reload = () => run(async () => { update(await api.reload(revision)); setName(w.name); });
  const requestReload = () => dirty ? confirm("Discard your unsaved name and reload from disk?", reload) : reload();
  return <>
    <div className="page-heading"><div><p className="eyebrow">WORKSPACE OVERVIEW</p><h1>{w.name}</h1><p className="subtitle">A local home for your infrastructure.</p></div><span className={`status-pill ${w.external_changed ? "warning" : ""}`}><i />{w.external_changed ? "Changes on disk" : w.read_only ? "Read-only" : "Up to date"}</span></div>
    {w.external_changed && <section className="notice conflict" role="alert"><div><strong>This workspace changed outside Ghosh.</strong><p>{dirty ? "Your unsaved name is still here. Reload to review the files before saving." : "Reload to validate the new files. Saving is paused until then."}</p></div><button disabled={busy} onClick={requestReload}>Reload workspace</button></section>}
    {w.read_only && <section className="notice" role="status"><strong>This package uses schema {w.schema}.</strong><p>Open it with a newer version of Ghosh to make changes.</p></section>}
    <div className="summary-grid"><div className="stat"><span className="stat-label">CONFIGURED NODES</span><strong>{w.nodes.length.toString().padStart(2, "0")}</strong><span>Definitions in this workspace</span></div><div className="stat"><span className="stat-label">LOCAL APPROVAL</span><strong className="word-stat">{w.trusted ? "Approved" : "Paused"}</strong><span>{w.trusted ? "Local execution can be reviewed" : "Resume to review operations"}</span></div><div className="stat"><span className="stat-label">STORAGE</span><strong className="word-stat">Local</strong><span>Portable .ghosh package</span></div></div>
    <section className="panel"><div className="section-heading"><div><h2>Workspace details</h2><p>Changes stay in your local package.</p></div>{dirty && <span className="unsaved">Unsaved changes</span>}</div>
      <form id="workspace-details-form" onSubmit={e => { e.preventDefault(); if (!busy && dirty && name.trim() && !w.read_only && !w.external_changed) run(async () => update(await api.rename({ ...revision, name: name.trim() }))); }}>
        <label htmlFor="workspace-name">Workspace name</label><div className="input-row"><input id="workspace-name" maxLength={120} value={name} onChange={e => setName(e.target.value)} disabled={busy || w.read_only} /><button className="primary" disabled={busy || !dirty || !name.trim() || w.read_only || w.external_changed}><Icon name="FileCode" />Save changes</button></div>
        <div className="detail-line"><span>Location</span><code>{w.path}</code></div>
      </form>
    </section>
    <section className="panel"><div className="section-heading"><div><h2>Nodes <span className="count">{w.nodes.length}</span></h2><p>Your workspace inventory. No connections are started automatically.</p></div></div>
      {w.nodes.length ? <div className="table-scroll"><table><thead><tr><th>Node</th><th>Address</th><th>User</th><th>Labels</th></tr></thead><tbody>{w.nodes.map(node => <tr key={node.id}><td><span className="node-dot" />{node.id}</td><td className="mono">{node.host}:{node.port}</td><td>{node.user}</td><td>{node.labels.length ? node.labels.map(label => <span className="tag" key={label}>{label}</span>) : <span className="muted">—</span>}</td></tr>)}</tbody></table></div> : <div className="empty-state"><span className="empty-icon"><Icon name="Server" /></span><h3>No nodes yet</h3><p>Add a node from Nodes or the command popup.</p></div>}
    </section>
    <section className="trust-panel"><div className="trust-symbol"><Icon name="ShieldCheck" /></div><div><h3>{w.trusted ? "Locally approved" : "Workspace operations paused"}</h3><p>Local workspaces are approved by default. Opening runs nothing. SSH server identity and each execution are checked separately.</p></div><button disabled={busy || w.read_only || w.external_changed} onClick={() => run(async () => update(await api.trust({ ...revision, trusted: !w.trusted })))}>{w.trusted ? "Pause operations" : "Resume operations"}</button></section>
    <details className="diagnostics"><summary>Workspace status & details</summary><div className="detail-line"><span>Workspace ID</span><code>{w.id}</code></div><div className="detail-line"><span>Revision</span><code>{w.revision}</code></div>{w.notices.map(notice => <p key={notice}>{notice}</p>)}<button disabled={busy} onClick={requestReload}>Reload from disk</button></details>
  </>;
}
