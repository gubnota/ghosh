import { useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../../app/ipc";
import { Select } from "../../components/Select";
import { ConnectionTest } from "../nodes/ConnectionTest";
import type { MonitorPacket, MonitorSession, WorkspaceSnapshot } from "../../generated/contracts";

const presets = [
  { label: "Journal · recent", command: "journalctl -n 100 --no-pager -o short-iso" },
  { label: "Journal · follow", command: "journalctl -n 50 -f --no-pager -o short-iso" },
  { label: "File · follow syslog", command: "tail -n 100 -f /var/log/syslog" },
  { label: "Kernel · recent", command: "dmesg | tail -n 100" },
  { label: "macOS · recent", command: "/usr/bin/log show --last 5m --style compact --info" },
  { label: "macOS · follow", command: "/usr/bin/log stream --style compact --level info" },
];
export function MonitorView({ workspace, active, busyChanged, modalChanged }: { workspace: WorkspaceSnapshot; active: boolean; busyChanged: (busy: boolean) => void; modalChanged: (open: boolean) => void }) {
  const [target, setTarget] = useState("local"), [session, setSession] = useState<MonitorSession | null>(null);
  const [packet, setPacket] = useState<MonitorPacket | null>(null), [command, setCommand] = useState(presets[0].command);
  const [connecting, setConnecting] = useState(false), [busy, setBusy] = useState(false), [error, setError] = useState("");
  const current = useRef<MonitorSession | null>(null), visible = useRef(active), generation = useRef(0);
  visible.current = active;
  const unavailable = workspace.read_only || workspace.external_changed || !workspace.trusted;
  useEffect(() => { busyChanged(busy); return () => busyChanged(false); }, [busy, busyChanged]);
  useEffect(() => { modalChanged(connecting); return () => modalChanged(false); }, [connecting, modalChanged]);
  const disconnect = async () => {
    generation.current++; const owned = current.current; current.current = null; setSession(null);
    if (owned) await api.monitorClose(owned.id);
  };
  useEffect(() => {
    generation.current++; const owned = current.current; current.current = null; setSession(null); setPacket(null); setConnecting(false);
    if (owned) void api.monitorClose(owned.id).catch(() => {});
    return () => { generation.current++; const owned = current.current; current.current = null; if (owned) void api.monitorClose(owned.id).catch(() => {}); };
  }, [workspace.id, workspace.revision, unavailable]);
  const attach = (opened: MonitorSession) => { current.current = opened; setSession(opened); setPacket(null); setError(""); };
  useEffect(() => {
    if (!session) return;
    let disposed = false; let timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      try { const result = await api.monitorPoll(session.id, visible.current && !document.hidden); if (!disposed) setPacket(result); }
      catch (e) { if (!disposed) { setError(errorMessage(e)); void disconnect().catch(() => {}); } return; }
      if (!disposed) timer = setTimeout(() => { void poll(); }, 250);
    };
    void poll(); return () => { disposed = true; clearTimeout(timer); };
  }, [session?.id]);
  const localConnect = async () => {
    const epoch = generation.current; setBusy(true); setError("");
    try { const opened = await api.monitorOpen({ context: { workspace_id: workspace.id, revision: workspace.revision }, connection: null }); if (epoch !== generation.current) await api.monitorClose(opened.id); else attach(opened); }
    catch (e) { setError(errorMessage(e)); } finally { setBusy(false); }
  };
  const run = async () => { if (!session) return; setBusy(true); setError(""); try { await api.monitorStart(session.id, command); setPacket(value => value && { ...value, running: true, output: "", exit_code: null }); } catch (e) { setError(errorMessage(e)); } finally { setBusy(false); } };
  const percent = (value: number | null | undefined) => value == null ? "Unavailable" : `${value.toFixed(1)}%`;
  return <>
    <div className="page-heading"><div><p className="eyebrow">NODE OBSERVABILITY</p><h1>Monitor</h1><p>Resource use and live logs on an independent connection.</p></div></div>
    {error && <p role="alert" className="form-error">{error}</p>}
    <section className="panel"><label htmlFor="monitor-target">Target</label><Select id="monitor-target" value={target} disabled={busy || connecting} onChange={e => { void disconnect().catch(e => setError(errorMessage(e))); setTarget(e.target.value); setPacket(null); }}><option value="local">Local machine</option>{workspace.nodes.map(node => <option key={node.id} value={node.id}>{node.id} · {node.user}@{node.host}</option>)}</Select>
      <div className="inline-actions">{session ? <><span role="status">Connected · {session.target}</span><button disabled={busy} onClick={() => { void disconnect().catch(e => setError(errorMessage(e))); }}>Stop and disconnect</button></> : <button disabled={busy || connecting || unavailable} onClick={() => { if (target === "local") void localConnect(); else setConnecting(true); }}>Connect Monitor</button>}</div>
      {unavailable && <p className="field-help">Trust and reload the workspace in Overview before connecting.</p>}
    </section>
    <section className="panel monitor-resources" aria-label="Node resource consumption">{([['CPU', packet?.metrics.cpu_percent], ['Memory', packet?.metrics.memory_percent], ['Disk /', packet?.metrics.disk_percent]] as const).map(([label, value]) => <div key={label}><h2>{label}</h2><strong>{percent(value)}</strong>{value != null && <meter min={0} max={100} value={value} aria-label={`${label} consumption`} />}</div>)}</section>
    <section className="panel"><h2>Logs</h2><label htmlFor="monitor-preset">Command preset</label><Select id="monitor-preset" value="" disabled={busy || packet?.running} onChange={e => setCommand(e.target.value)}><option value="" disabled>Choose a log command…</option>{presets.map(preset => <option key={preset.label} value={preset.command}>{preset.label}</option>)}</Select><label htmlFor="monitor-command">Log command</label><textarea id="monitor-command" spellCheck={false} maxLength={16384} value={command} disabled={busy || packet?.running} onChange={e => setCommand(e.target.value)} /><div className="inline-actions"><button disabled={!session || unavailable || busy || packet?.running || !command.trim()} onClick={() => { void run(); }}>Run log command</button>{packet?.running && session && <button onClick={() => { void disconnect().catch(e => setError(errorMessage(e))); }}>Stop</button>}</div><p className="field-help">Commands run only when requested. Tools and log permissions depend on the target. Stop closes this connection; remote command termination depends on its SSH server. Resource probes pause while Monitor is hidden.</p>
      {packet && <><p role="status">{packet.running ? "Running…" : packet.exit_code != null ? `Finished · exit ${packet.exit_code}` : "Ready"}{packet.truncated ? " · Showing latest 64 KiB" : ""}</p><pre className="monitor-output" aria-label="Monitor output">{packet.output || "No output yet."}</pre></>}
    </section>
    {connecting && <ConnectionTest workspace={workspace} target={target} mode="monitor" onMonitor={attach} dismiss={() => setConnecting(false)} busyChanged={busyChanged} />}
  </>;
}
