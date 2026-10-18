import { useEffect, useState } from "react";
import type { ApprovalDecision, ApprovalSnapshot } from "../../generated/contracts";
import type { Operations } from "../../app/useOperations";
import { Dialog } from "../../components/Dialog";
import { readable, timestamp } from "./JobsView";

export function ApprovalsView({ operations, modalChanged }: { operations: Operations; modalChanged: (open: boolean) => void }) {
  const [selected, setSelected] = useState<ApprovalSnapshot | null>(null);
  const [query, setQuery] = useState("");
  useEffect(() => { modalChanged(!!selected); return () => modalChanged(false); }, [!!selected, modalChanged]);
  const requests = operations.snapshot?.approvals.requests;
  const current = requests?.find(request => request.id === selected?.id);
  const review = current || selected;
  const pending = !!current && current.state === "pending";
  const granted = !!current && ["allowed_once", "allowed_session"].includes(current.state);
  const live = pending || granted;
  const decide = async (decision: ApprovalDecision) => {
    if (!review) return;
    await operations.decide({ id: review.id, decision });
    requestAnimationFrame(() => document.getElementById("approval-title")?.closest("dialog")?.querySelector<HTMLButtonElement>("[data-dialog-focus]")?.focus());
  };
  return <><div className="page-heading"><div><p className="eyebrow">SYSTEM</p><h1>Approvals</h1><p>Review the exact operation before permitting it.</p></div><button disabled={operations.mutating} onClick={() => { void operations.refresh(); }}>Refresh</button></div>
    {!selected && operations.error && <p className="notice error" role="alert">{operations.error}</p>}
    <section className="panel"><label htmlFor="approvals-search">Filter approvals</label><input id="approvals-search" data-surface-search value={query} onChange={e => setQuery(e.target.value)} placeholder="Tool, requester, target or state" />
      {!requests ? <p role="status">Loading native approvals…</p> : !requests.length ? <div className="empty-state"><h2>No approval requests</h2><p>Requests appear when a native execution service asks to perform an operation. Opening this inbox starts nothing.</p></div> : <div className="operation-list">{requests.filter(request => `${request.binding.tool} ${request.binding.requester} ${request.binding.targets.join(" ")} ${request.state}`.toLowerCase().includes(query.toLowerCase())).map(request => <article className="operation-card" key={request.id}><div className="section-heading"><h2>{request.binding.tool}</h2><span className={`operation-state state-${request.state}`}>{readable(request.state)}</span></div><p>{request.binding.requester} · {request.binding.targets.join(", ")}</p><button onClick={() => setSelected(request)}>Review {request.binding.tool}</button></article>)}</div>}
      <p className="field-help">Grants expire after five minutes. Configuration, workspace and trust changes invalidate them. Session grants apply only to the exact reviewed content and targets.</p>
    </section>
    {!!operations.snapshot?.approvals.audit.length && <section className="panel"><h2>Session audit</h2><ol className="audit-list">{operations.snapshot.approvals.audit.map((event, index) => <li key={`${event.request_id}-${event.at}-${index}`}><time>{timestamp(event.at)}</time> · {readable(event.kind)}<small>{event.request_id}</small></li>)}</ol><p className="field-help">Up to 256 events are kept in memory; they do not survive app restart.</p></section>}
    {review && <Dialog title="approval-title" dismiss={() => setSelected(null)} busy={operations.mutating} alert><p className="eyebrow">AUTHORIZE OPERATION</p><h2 id="approval-title">{review.binding.tool}</h2><p>Status: {current ? readable(current.state) : "No longer available"}</p><dl className="operation-details"><dt>Requester</dt><dd>{review.binding.requester}</dd><dt>Targets</dt><dd>{review.binding.targets.join(", ")}</dd><dt>Workspace</dt><dd>{review.binding.context.workspace_id}</dd><dt>Configuration revision</dt><dd>{review.binding.context.revision}</dd><dt>Source SHA-256</dt><dd>{review.binding.source_digest}</dd><dt>Route SHA-256</dt><dd>{review.binding.route_digest}</dd><dt>Identity SHA-256</dt><dd>{review.binding.identity_digest}</dd><dt>Session</dt><dd>{review.binding.context.session_id}</dd><dt>Expires</dt><dd>{timestamp(review.expires_at)}</dd></dl><p className="field-help">Allow once permits one matching invocation. Allow for session permits matching invocations until expiry or invalidation.</p>{operations.error && <p role="alert" className="notice error">{operations.error}</p>}<div className="modal-actions approval-actions"><button autoFocus data-dialog-focus disabled={operations.mutating} onClick={() => setSelected(null)}>Cancel</button>{pending && <><button disabled={operations.mutating} onClick={() => { void decide("deny"); }}>Deny</button><button disabled={operations.mutating} onClick={() => { void decide("allow_once"); }}>Allow once</button><button className="primary" disabled={operations.mutating} onClick={() => { void decide("allow_session"); }}>Allow for session</button></>}{granted && <button disabled={operations.mutating} onClick={() => { void decide("revoke"); }}>Revoke grant</button>}</div>{!live && <p role="status">This request cannot authorize an operation.</p>}</Dialog>}
  </>;
}
