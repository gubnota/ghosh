import { useId, useState } from "react";
import { Icon } from "../../components/Icon";
import type { RecentWorkspace } from "../../generated/contracts";

export function RecentWorkspaces({ entries, busy, open, forget }: {
  entries: RecentWorkspace[]; busy: boolean; open: (path: string) => void;
  forget: (path: string | null) => void;
}) {
  const [search, setSearch] = useState("");
  const searchId = useId();
  const visible = entries.filter(entry => `${entry.name} ${entry.path}`.toLowerCase().includes(search.toLowerCase()));
  return <section className="recent-workspaces">
    <div className="section-heading"><h2>Recent workspaces</h2><button disabled={busy || !entries.length} onClick={() => forget(null)}>Clear history</button></div>
    <label className="sr-only" htmlFor={searchId}>Find a workspace</label>
    <input id={searchId} placeholder="Find a workspace…" value={search} onChange={event => setSearch(event.target.value)} />
    <div className="recent-grid">{visible.map(entry => <div className="recent-entry" key={entry.path}>
      <button className="recent-card" disabled={busy} onClick={() => open(entry.path)} title={entry.path}><span className="workspace-avatar">{entry.name.slice(0, 1).toUpperCase()}</span><span><strong>{entry.name}</strong><small>{entry.path}</small></span><Icon name="ArrowUpRight" /></button>
      <button className="quiet" disabled={busy} aria-label={`Remove ${entry.name} from history`} onClick={() => forget(entry.path)}><Icon name="X" /></button>
    </div>)}</div>
    {!visible.length && <p className="field-help">{entries.length ? "No matching workspaces." : "No recent workspaces."}</p>}
    <p className="field-help">Removing history leaves workspace files intact. Clearing history also turns off automatic restoration until you open a workspace again.</p>
  </section>;
}
