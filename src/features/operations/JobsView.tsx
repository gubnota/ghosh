import { useState } from "react";
import type { JobKind } from "../../generated/contracts";
import type { Operations } from "../../app/useOperations";

const labels: Record<JobKind, string> = { automation_run: "Run automation", node_connect: "Test SSH connection", inventory_save: "Save node inventory", workspace_open: "Open workspace", workspace_create: "Create workspace", workspace_save: "Save workspace", workspace_reload: "Reload workspace", workspace_trust: "Change workspace trust" };
export const readable = (value: string) => value.replaceAll("_", " ");
export const timestamp = (value: number) => new Date(value).toLocaleString();
export function JobsView({ operations }: { operations: Operations }) {
  const [query, setQuery] = useState("");
  const jobs = operations.snapshot?.jobs.jobs;
  const visible = jobs?.filter(job => `${labels[job.kind]} ${job.state} ${job.id}`.toLowerCase().includes(query.toLowerCase()));
  return <><div className="page-heading"><div><p className="eyebrow">SYSTEM</p><h1>Background jobs</h1><p>Native operations in this app session. Most recent first.</p></div><button disabled={operations.mutating} onClick={() => { void operations.refresh(); }}>Refresh</button></div>
    {operations.error && <p className="notice error" role="alert">{operations.error}</p>}
    <section className="panel"><label htmlFor="jobs-search">Filter jobs</label><input id="jobs-search" data-surface-search value={query} onChange={e => setQuery(e.target.value)} placeholder="Operation, state or ID" />
      {!jobs ? <p role="status">Loading native jobs…</p> : !visible?.length ? <div className="empty-state"><h2>{jobs.length ? "No matching jobs" : "No jobs yet"}</h2><p>Opening, creating or saving a workspace records a real operation here.</p></div> : <div className="operation-list">{visible.map(job => <article className="operation-card" key={job.id}><div className="section-heading"><h2>{labels[job.kind]}</h2><span className={`operation-state state-${job.state}`}>{readable(job.state)}</span></div><p className="field-help">{timestamp(job.started_at)}{job.finished_at !== null && ` · Finished ${timestamp(job.finished_at)}`}</p><details><summary>Operation details</summary><dl className="operation-details"><dt>Job ID</dt><dd>{job.id}</dd><dt>Stages</dt><dd>{job.stages.map(readable).join(" → ")}</dd>{job.error && <><dt>Error code</dt><dd>{job.error}</dd></>}</dl></details>{["running", "cancelling"].includes(job.state) && (job.cancellable ? <button disabled={operations.mutating || job.state === "cancelling"} onClick={() => { void operations.cancel(job.id); }}>{job.state === "cancelling" ? "Cancellation requested" : "Cancel job"}</button> : <p className="field-help">This short operation must finish before it can be closed.</p>)}</article>)}</div>}
      <p className="field-help">Up to 128 jobs are kept in memory. Cancellation requests a stop; completed changes remain applied.</p>
    </section></>;
}
