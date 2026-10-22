import { Icon } from "../../components/Icon";
import { useEffect, useMemo, useRef, useState } from "react";
import { desktop } from "../../app/ipc";
import { NodeEditor, type NodeAction } from "./NodeEditor";
import { ConnectionTest } from "./ConnectionTest";
import type { WorkspaceSnapshot, TerminalSession } from "../../generated/contracts";

export function NodesView({ workspace, busy, runActive = false, request, requestHandled, modalChanged, busyChanged, update, onTerminal }: { workspace: WorkspaceSnapshot; busy: boolean; runActive?: boolean; request: { action: "create" | "edit" | "test" | "terminal"; sequence: number } | null; requestHandled: () => void; modalChanged: (open: boolean) => void; busyChanged: (busy: boolean) => void; update: (workspace: WorkspaceSnapshot) => void; onTerminal: (session: TerminalSession) => void }) {
  const [editor, setEditor] = useState<{ action: NodeAction; target?: string } | null>(null);
  const [connection, setConnection] = useState<{ target?: string; mode: "test" | "terminal" } | null>(null);
  const editable = desktop && !workspace.read_only && !workspace.external_changed && !busy;
  const handled = useRef(0);
  useEffect(() => { modalChanged(!!editor || !!connection); return () => modalChanged(false); }, [editor, connection, modalChanged]);
  useEffect(() => {
    if (request && request.sequence > handled.current) {
      handled.current = request.sequence;
      requestHandled();
      if (editable) {
        if (runActive && !["test", "terminal"].includes(request.action)) return;
        if (request.action === "test" || request.action === "terminal") setConnection({ mode: request.action }); else setEditor({ action: request.action });
      }
    }
  }, [request, editable, requestHandled]);
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState<Set<string>>(new Set());
  useEffect(() => { setSelected(previous => new Set([...previous].filter(id => workspace.nodes.some(node => node.id === id)))); }, [workspace.nodes]);
  const visible = useMemo(() => workspace.nodes.filter(node => [node.id, node.host, node.user, ...node.labels].join(" ").toLowerCase().includes(query.trim().toLowerCase())), [workspace.nodes, query]);
  const selectAll = useRef<HTMLInputElement>(null);
  const visibleSelected = visible.filter(node => selected.has(node.id)).length;
  useEffect(() => { if (selectAll.current) selectAll.current.indeterminate = visibleSelected > 0 && visibleSelected < visible.length; }, [visibleSelected, visible.length]);
  const toggle = (id: string) => setSelected(previous => { const next = new Set(previous); if (!next.delete(id)) next.add(id); return next; });
  return <><div className="page-heading"><div><p className="eyebrow">INFRASTRUCTURE</p><h1>Nodes <span className="count">{workspace.nodes.length}</span></h1><p>Configured inventory. No connection starts automatically.</p></div><div className="inline-actions"><button disabled={!editable || runActive} onClick={() => setEditor({ action: "import" })}><Icon name="Upload" />Import TOML</button><button className="primary" disabled={!editable || runActive} onClick={() => setEditor({ action: "create" })}><Icon name="Plus" />Add node</button></div></div>
    <section className="panel"><div className="table-toolbar"><label htmlFor="node-search" className="sr-only">Filter nodes</label><input id="node-search" data-surface-search placeholder="Filter by name, host, user or label…" value={query} onChange={e => setQuery(e.target.value)} /><span role="status">{selected.size} selected</span><button disabled={!selected.size} onClick={() => setSelected(new Set())}>Clear selection</button></div>
      {visible.length ? <div className="table-scroll"><table><thead><tr><th><div className="node-identity"><input ref={selectAll} type="checkbox" aria-label="Select all visible nodes" checked={visible.every(node => selected.has(node.id))} onChange={e => setSelected(previous => { const next = new Set(previous); for (const node of visible) { if (e.target.checked) next.add(node.id); else next.delete(node.id); } return next; })} /><span>NODE</span></div></th><th>Address</th><th>User</th><th>Labels</th><th>Actions</th></tr></thead><tbody>{visible.map(node => <tr key={node.id} className={selected.has(node.id) ? "row-selected" : ""}><td><div className="node-identity"><input type="checkbox" aria-label={`Select ${node.id}`} checked={selected.has(node.id)} onChange={() => toggle(node.id)} /><span className="node-dot" title="Configured · not monitored" aria-label="Configured · not monitored" /><span>{node.id}</span></div></td><td>{node.host}:{node.port}</td><td>{node.user}</td><td>{node.labels.map(label => <span className="tag" key={label}>{label}</span>)}</td><td><div className="inline-actions"><button disabled={!editable || !workspace.trusted} aria-label={`Test ${node.id}`} onClick={() => setConnection({ target: node.id, mode: "test" })}><Icon name="Play" />Test</button><button disabled={!editable || !workspace.trusted} aria-label={`Terminal ${node.id}`} onClick={() => setConnection({ target: node.id, mode: "terminal" })}><Icon name="Terminal" />Terminal</button><button disabled={!editable || runActive} aria-label={`Edit ${node.id}`} onClick={() => setEditor({ action: "edit", target: node.id })}><Icon name="Pencil" />Edit</button><button disabled={!editable || runActive} aria-label={`Remove ${node.id}`} onClick={() => setEditor({ action: "remove", target: node.id })}><Icon name="Trash2" />Remove</button></div></td></tr>)}</tbody></table></div> : <div className="empty-state"><h3>{query ? "No matching nodes" : "No nodes yet"}</h3><p>{query ? "Try a different name, host or label." : "Add a node or import a TOML definition to begin."}</p></div>}
    </section><p className="field-help">{workspace.external_changed ? "Reload the workspace from Overview to edit inventory." : workspace.read_only ? "This workspace is read-only." : "Inventory changes are reviewed before saving. Test verifies SSH login; Terminal opens a real remote shell. Direct connections support file keys, an SSH agent, or a password entered for this connection."}</p>
    {editor && <NodeEditor workspace={workspace} action={editor.action} target={editor.target} dismiss={() => setEditor(null)} update={update} busyChanged={busyChanged} />}
    {connection && <ConnectionTest workspace={workspace} target={connection.target} mode={connection.mode} onTerminal={onTerminal} dismiss={() => setConnection(null)} busyChanged={busyChanged} />}
  </>;
}
