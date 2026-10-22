import { useEffect, useRef, useState } from "react";
import { Dialog } from "../../components/Dialog";
import { api, errorMessage } from "../../app/ipc";
import { readable, timestamp } from "../operations/JobsView";
import { runPercent, runActivity, useRuns } from "./useRuns";
import type { RunReport, WorkspaceSnapshot } from "../../generated/contracts";

export function RunsView({ workspace }: { workspace: WorkspaceSnapshot }) {
  const { reports, error: loadError } = useRuns(workspace);
  const [cleanup, setCleanup] = useState<RunReport | "finished" | null>(null), [removing, setRemoving] = useState(false);
  const [error, setError] = useState("");
  const [query, setQuery] = useState(""), [stopping, setStopping] = useState<string[]>([]);
  const remove = async () => {
    if (!cleanup || removing) return; setRemoving(true); setError("");
    try { await api.runsCleanup({ context: { workspace_id: workspace.id, revision: workspace.revision }, id: cleanup === "finished" ? null : cleanup.id }); setCleanup(null); }
    catch (e) { setError(errorMessage(e)); } finally { setRemoving(false); }
  };
  const download = (report: RunReport) => {
    const url = URL.createObjectURL(new Blob([JSON.stringify(report, null, 2)], { type: "application/json" }));
    const link = document.createElement("a"); link.href = url; link.download = `ghosh-run-${report.id}.json`; link.click();
    window.setTimeout(() => URL.revokeObjectURL(url), 1000);
  };
  const stop = async (id: string) => {
    setStopping(current => [...current, id]);
    try { await api.runStop(id); } catch (e) { setError(errorMessage(e)); setStopping(current => current.filter(value => value !== id)); }
  };
  return <><div className="page-heading"><div><p className="eyebrow">EXECUTION HISTORY</p><h1>Runs</h1><p>Live logs and saved results. Runs continue while you use other sections. Targets run one at a time.</p></div><button disabled={!reports.some(report => report.state !== "running") || workspace.read_only || workspace.external_changed} onClick={() => setCleanup("finished")}>Clear finished runs</button></div>
    {(error || loadError) && <p role="alert" className="form-error">{error || loadError}</p>}
    <section className="panel"><label htmlFor="runs-search">Filter runs</label><input id="runs-search" data-surface-search value={query} onChange={e => setQuery(e.target.value)} placeholder="Scenario, target or state…" />
      {!reports.length && <div className="empty-state"><h2>No runs yet</h2><p>Choose Run in Playbooks, select targets and start.</p></div>}
      <div className="operation-list">{reports.filter(report => `${report.automation_id} ${report.targets.join(" ")} ${report.state}`.toLowerCase().includes(query.toLowerCase())).map(report => <article className="operation-card" key={report.id}>
        <div className="section-heading"><h2 title={report.automation_id}>{report.automation_id}</h2><span className={`operation-state state-${report.state}`}>{report.mode === "check" && report.state === "succeeded" ? report.results.some(result => ["drift", "changed"].includes(result.state)) ? "Check complete · changes needed" : "Check complete · no changes reported" : readable(report.state)}</span></div>
        <p className="field-help">{timestamp(report.started_at)} · {report.mode} · {report.targets.map(id => id === "@local" ? "Local machine" : id).join(", ")}</p><p>{report.message}</p>
        <RunLive report={report} />
        {report.state === "running" && <button disabled={stopping.includes(report.id)} onClick={() => { void stop(report.id); }}>{stopping.includes(report.id) ? "Stop requested" : "Stop run"}</button>}
        {report.state !== "running" && <button disabled={workspace.read_only || workspace.external_changed} onClick={() => setCleanup(report)}>Remove run</button>}
        <button onClick={() => download(report)}>Export results · JSON</button>
        <details open={report.state === "running" || undefined}><summary>{report.results.length} task phase(s) · Results</summary>
          <div className="run-results">{report.results.map((result, index) => <section key={index}><div className="section-heading"><h3>{result.node_id} · {result.name} · {result.phase}</h3><span className={`operation-state state-${result.state}`}>{readable(result.state)}</span></div>{result.exit_code !== null && <p className="field-help">Exit {result.exit_code}</p>}{result.output && <pre tabIndex={0}>{result.output}</pre>}{result.truncated && <p className="field-help">Output truncated to the report limit.</p>}</section>)}</div>
        </details><p className="field-help">Run {report.id} · Revision {report.revision.slice(0, 12)}</p>
      </article>)}</div><p className="field-help">Latest 100 runs. Local and SSH output streams during commands; Ansible publishes task events. Output is limited to 32 KiB per phase and 256 KiB per run. Interrupted remote commands may still be running; inspect the node before retrying.</p>
    </section>
    {cleanup && <Dialog title="cleanup-title" busy={removing} dismiss={() => setCleanup(null)}><h2 id="cleanup-title">{cleanup === "finished" ? "Clear finished run history?" : "Remove this run?"}</h2><p>{cleanup === "finished" ? `Remove all finished run reports and event logs for ${workspace.name}, including records beyond the latest 100. Running runs are retained.` : `Remove ${cleanup.automation_id} (${cleanup.id}) and its saved event log.`}</p><p>This removes local history. Changes already made on targets remain.</p>{error && <p role="alert">{error}</p>}<div className="modal-actions"><button disabled={removing} onClick={() => setCleanup(null)}>Cancel</button><button disabled={removing} onClick={() => { void remove(); }}>Remove history</button></div></Dialog>}
    </>;
}

function RunLive({ report }: { report: RunReport }) {
  const [follow, setFollow] = useState(true);
  const log = useRef<HTMLPreElement>(null), progress = report.progress;
  useEffect(() => { if (follow && log.current) log.current.scrollTop = log.current.scrollHeight; }, [progress?.output, follow]);
  if (!progress) return null;
  const percent = runPercent(report);
  return <div className="run-live">
    <div className="section-heading"><strong>{percent === undefined ? `${progress.completed_steps} completed task(s)` : `${runActivity(report)} · ${progress.completed_steps} / ${progress.total_steps} completed${report.state === "running" ? "" : ` · ${percent}%`}`}</strong>{report.state === "running" && <span className="operation-state">LIVE</span>}</div>
    <progress aria-label={`${report.automation_id} overall progress`} max={100} value={report.state === "running" ? undefined : percent} />
    <p className="field-help">{report.state === "running" ? "Current" : "Last"}: {progress.node_id === "@local" ? "Local machine" : progress.node_id} · {progress.step} {progress.phase && `· ${progress.phase}`}</p>
    {report.state === "running" && <><label className="checkbox-label"><input type="checkbox" checked={follow} onChange={event => setFollow(event.target.checked)} />Follow live output</label><pre ref={log} className="live-output" tabIndex={0} aria-label="Live command output">{progress.output || "Waiting for output…"}</pre>{progress.truncated && <p className="field-help">Live preview capped at 32 KiB. Both streams continue to drain.</p>}<p className="field-help">You can switch sections and return here. Keep Ghosh open. Input values are masked; a partial line may wait for more bytes.</p></>}
  </div>;
}
