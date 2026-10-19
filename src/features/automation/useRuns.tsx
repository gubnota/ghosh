import { useEffect, useState } from "react";
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
export function BackgroundRun({ workspace, open }: { workspace: WorkspaceSnapshot | null; open: () => void }) {
  const { reports } = useRuns(workspace), report = reports.find(value => value.state === "running");
  if (!report) return null;
  const percent = runPercent(report);
  return <button className="quiet background-run" onClick={open} title={`${report.automation_id} · ${report.progress?.step || "Connecting"}`} aria-label={`Running ${report.automation_id}. Open live logs`}><span className="live-dot" />{percent === undefined ? "Running" : `${percent}%`} · <span>{report.automation_id}</span></button>;
}
