import { useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../../app/ipc";
import type { RunReport } from "../../generated/contracts";
import { runPercent, runActivity } from "./useRuns";

export function RunMonitor({ report, minimize, open }: { report: RunReport; minimize: () => void; open: () => void }) {
  const log = useRef<HTMLPreElement>(null);
  const [follow, setFollow] = useState(true), [stopping, setStopping] = useState(false), [error, setError] = useState("");
  useEffect(() => { if (follow && log.current) log.current.scrollTop = log.current.scrollHeight; }, [report.progress?.output, follow]);
  useEffect(() => { setStopping(false); setError(""); }, [report.id]);
  const progress = report.progress, percent = runPercent(report);
  return <aside className="run-monitor" aria-label="Live run monitor">
    <div className="section-heading"><strong>{report.automation_id}</strong><button onClick={minimize} aria-label="Minimize run monitor">Minimize</button></div>
    <p>{runActivity(report)} · {progress?.node_id === "@local" ? "Local machine" : progress?.node_id} · {progress?.step || "Connecting"}</p>
    <progress aria-label="Compact run progress" max={100} value={report.state === "running" ? undefined : percent} />
    <label className="checkbox-label"><input type="checkbox" checked={follow} onChange={e => setFollow(e.target.checked)} />Follow output</label>
    <pre ref={log} tabIndex={0} aria-label="Compact live output">{progress?.output || report.results.at(-1)?.output || report.message}</pre>
    {progress?.truncated && <p className="field-help">Output preview capped. Full bounded results are in Runs.</p>}
    {error && <p role="alert">{error}</p>}
    <div className="inline-actions">{report.state === "running" && <button disabled={stopping} onClick={() => { setStopping(true); void api.runStop(report.id).catch(e => { setStopping(false); setError(errorMessage(e)); }); }}>{stopping ? "Stop requested" : "Stop background run"}</button>}<button onClick={() => { minimize(); open(); }}>Open full run</button></div>
    <p className="field-help">Use other sections or the SSH shell while this runs. Keep Ghosh open.</p>
  </aside>;
}
