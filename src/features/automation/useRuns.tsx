import { useEffect, useState } from "react";
import { RunMonitor } from "./RunMonitor";
import { api, errorMessage } from "../../app/ipc";
import type { RunReport, WorkspaceSnapshot } from "../../generated/contracts";

export function useRuns(workspace: WorkspaceSnapshot | null) {
  const [reports, setReports] = useState<RunReport[]>([]), [error, setError] = useState("");
  useEffect(() => {
    let alive = true, pending = false;
    setReports([]); setError("");
    if (!workspace) return;
    const refresh = async () => {
      if (pending || !alive) return; pending = true;
      try {
        const next = await api.runs({ workspace_id: workspace.id, revision: workspace.revision });
        if (alive) { setReports(next); setError(""); }
      } catch (e) { if (alive) setError(errorMessage(e)); } finally { pending = false; }
    };
    void refresh(); const timer = window.setInterval(() => { void refresh(); }, 600);
    return () => { alive = false; window.clearInterval(timer); };
  }, [workspace?.id, workspace?.revision]);
  return { reports, error };
}
export function runPercent(report: RunReport) {
  if (!report.progress?.total_steps) return undefined;
  return report.state === "succeeded" ? 100 : Math.min(report.state === "running" ? 99 : 100, Math.floor(100 * report.progress.completed_steps / report.progress.total_steps));
}
export function runActivity(report: RunReport) {
  const p = report.progress;
  return report.state === "running" ? p?.total_steps ? `Running step ${Math.min(p.completed_steps + 1, p.total_steps)} of ${p.total_steps}` : "Connecting" : report.state;
}
export function BackgroundRun({ workspace, open }: { workspace: WorkspaceSnapshot | null; open: () => void }) {
  const { reports } = useRuns(workspace), report = reports.find(value => value.state === "running");
  const [visible, setVisible] = useState(false), [last, setLast] = useState<RunReport | null>(null);
  useEffect(() => { setLast(null); setVisible(false); }, [workspace?.id]);
  useEffect(() => { if (report) { setLast(report); if (last?.id !== report.id) setVisible(true); } else if (last) { const finished = reports.find(value => value.id === last.id); if (!finished) { setLast(null); setVisible(false); } else if (finished !== last) setLast(finished); } }, [reports]);
  const shown = report || last;
  if (!shown) return null;
  return <>{visible && <RunMonitor report={shown} minimize={() => setVisible(false)} open={open} />}<button className="quiet background-run" onClick={() => setVisible(true)} title={`${shown.automation_id} · ${shown.progress?.step || "Connecting"}`} aria-label={`${shown.state === "running" ? "Running" : "Finished"} ${shown.automation_id}. Open live logs`}><span className="live-dot" />{shown.state !== "running" ? "Finished" : runActivity(shown)} · <span>{shown.automation_id}</span></button></>;
}
