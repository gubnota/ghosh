import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api, desktop, errorMessage } from "../../app/ipc";
import { Dialog } from "../../components/Dialog";
import { Icon } from "../../components/Icon";
import { Select } from "../../components/Select";
import type { AutomationDocument, AutomationKind, AutomationPreview, AutomationSummary, WorkspaceSnapshot } from "../../generated/contracts";
import { ComfyDialog } from "../comfy/ComfyDialog";
import { RunDialog } from "./RunDialog";
import { examples } from "./examples";
import { NotebookEditor, newNotebook } from "./NotebookEditor";
import { useRuns } from "./useRuns";
import { SourceEditor } from "./SourceEditor";

export type AutomationAction = "new" | "import" | "edit" | "check" | "apply";
type Draft = { document: AutomationDocument; replace: boolean; initial: string; remove?: boolean };
const labels: Record<AutomationKind, string> = { notebook: "Multistep Playbook", playbook: "Simple scenario / Ansible YAML", ansible: "Ansible project", shell: "Shell", python: "Python", deno: "Deno · JS / TS" };
const validId = (id: string) => /^[a-z0-9][a-z0-9_-]{0,63}$/.test(id);
export function AutomationView({ workspace, scripts, recipes = false, request, requestHandled, modalChanged, busyChanged, update, started }: {
  workspace: WorkspaceSnapshot; scripts: boolean; recipes?: boolean; newSteps: () => void;
  request: { action: AutomationAction; scripts: boolean; sequence: number } | null;
  requestHandled: () => void; modalChanged: (open: boolean) => void; busyChanged: (busy: boolean) => void;
  update: (workspace: WorkspaceSnapshot) => void; started: () => void;
}) {
  const [comfy, setComfy] = useState(false);
  const [items, setItems] = useState<AutomationSummary[]>([]), [query, setQuery] = useState("");
  const [loadedRevision, setLoadedRevision] = useState("");
  const [draft, setDraft] = useState<Draft | null>(null), [preview, setPreview] = useState<AutomationPreview | null>(null);
  const [busy, setBusy] = useState(false), [error, setError] = useState(""), [discard, setDiscard] = useState(false);
  const handled = useRef(0);
  const [restoreStatus, setRestoreStatus] = useState("");
  const [expanded, setExpanded] = useState(false);
  const [runItem, setRunItem] = useState<AutomationSummary | null>(null), [runMode, setRunMode] = useState<"check" | "apply">("check");
  const { reports } = useRuns(workspace);
  const [notebook, setNotebook] = useState<AutomationDocument | null>(null);
  const [runCells, setRunCells] = useState<string[]>([]);
  const editable = !workspace.read_only && !workspace.external_changed;
  const context = { workspace_id: workspace.id, revision: workspace.revision };
  const shown = items.filter(item => recipes ? item.id.startsWith("comfy-") : item.kind === "notebook");
  const offered = examples.filter(example => example.document.kind === "notebook");
  useEffect(() => {
    let alive = true;
    setLoadedRevision("");
    setNotebook(null);
    void api.automations(context).then(value => { if (alive) { setItems(value); setLoadedRevision(`${workspace.id}:${workspace.revision}`); } }).catch(e => { if (alive) setError(errorMessage(e)); });
    return () => { alive = false; };
  }, [workspace.id, workspace.revision]);
  useEffect(() => { modalChanged(!!draft || !!runItem || comfy); return () => modalChanged(false); }, [!!draft, !!runItem, comfy, modalChanged]);
  useEffect(() => { busyChanged(busy); return () => busyChanged(false); }, [busy, busyChanged]);
  const run = async (action: () => Promise<void>) => { setBusy(true); setError(""); try { await action(); } catch (e) { setError(errorMessage(e)); } finally { setBusy(false); } };
  const create = (document: AutomationDocument = scripts ? offered[0].document : { id: "multi-step", kind: "notebook", source: newNotebook }) => { setExpanded(document.kind === "notebook"); setError(""); setPreview(null); setDiscard(false); setDraft({ document: { ...document }, replace: false, initial: JSON.stringify(document) }); };
  const edit = (item: AutomationSummary, remove = false) => { void run(async () => {
    const document = await api.automationRead({ context, id: item.id, kind: item.kind });
    setExpanded(document.kind === "notebook" && !remove); setPreview(null); setDiscard(false); setDraft({ document, replace: true, initial: JSON.stringify(document), remove });
  }); };
  const importFile = (path: string | null = null) => { if (!editable || busy || draft || scripts) return; void run(async () => { const document = await api.playbookImport(path); if (document) create(document); }); };
  useEffect(() => {
    if (!desktop || scripts || recipes || draft || !editable || busy) return;
    let disposed = false; let unlisten: (() => void) | undefined;
    void getCurrentWindow().onDragDropEvent(event => { if (event.payload.type === "drop") { if (event.payload.paths.length === 1) importFile(event.payload.paths[0]); else setError("Drop one Playbook file at a time."); } }).then(fn => { if (disposed) fn(); else unlisten = fn; }).catch(e => { if (!disposed) setError(errorMessage(e)); });
    return () => { disposed = true; unlisten?.(); };
  }, [scripts, recipes, !!draft, editable, busy]);
  const exportItem = (item: AutomationSummary) => { void run(async () => { await api.playbookExport(await api.automationRead({ context, id: item.id, kind: item.kind })); }); };
  useEffect(() => {
    if (request && request.scripts === scripts && request.sequence > handled.current) {
      if (["edit", "check", "apply"].includes(request.action) && loadedRevision !== `${workspace.id}:${workspace.revision}`) return;
      handled.current = request.sequence; requestHandled();
      if (editable) { if (["check", "apply"].includes(request.action)) { const item = shown.find(item => item.validation.valid); if (item) { setRunItem(item); setRunMode(request.action === "check" ? "check" : "apply"); } else setError("Save a valid Playbook first."); } else if (request.action === "edit" && shown[0]) edit(shown[0]); else if (recipes) setComfy(true); else if (request.action === "import" && !scripts) importFile(); else create(); }
    }
  }, [request, editable, shown.length, loadedRevision, workspace.id, workspace.revision]);
  const dismiss = () => {
    if (busy) return;
    if (expanded) { setExpanded(false); return; }
    if (draft && JSON.stringify(draft.document) !== draft.initial && !discard) { setDiscard(true); return; }
    setDraft(null); setPreview(null); setDiscard(false); setError("");
  };
  const change = (document: AutomationDocument) => { if (draft) setDraft({ ...draft, document }); setPreview(null); setDiscard(false); };
  const mutation = draft ? { context, document: draft.document, replace: draft.replace, remove: !!draft.remove } : null;
  const idValid = !draft || !!draft.remove || validId(draft.document.id);
  const suggestedId = draft?.document.id.toLowerCase().replace(/[^a-z0-9_-]+/g, "-").replace(/^[-_]+/, "").slice(0, 64).replace(/-+$/, "") || "";
  const saveDraft = () => { if (mutation && idValid && !busy && editable) void run(async () => {
    const checked = await api.automationPreview(mutation); setPreview(checked);
    if (checked.validation.valid) { update(await api.automationCommit({ mutation, digest: checked.digest })); setDraft(null); setPreview(null); }
  }); };
  const restoreExamples = () => { void run(async () => {
    let current = workspace;
    const existing = new Set((await api.automations(context)).map(item => item.id));
    const restored: string[] = [], skipped: string[] = [];
    try {
      for (const example of offered) {
        const document = example.document;
        if (existing.has(document.id)) { skipped.push(document.id); continue; }
        const mutation = { context: { workspace_id: current.id, revision: current.revision }, document, replace: false, remove: false };
        const checked = await api.automationPreview(mutation);
        if (!checked.validation.valid) throw new Error(checked.validation.issues.join(" "));
        current = await api.automationCommit({ mutation, digest: checked.digest });
        update(current); existing.add(document.id); restored.push(document.id);
      }
    } finally {
      setRestoreStatus(`Restored: ${restored.join(", ") || "none"}. Kept existing: ${skipped.join(", ") || "none"}.`);
    }
  }); };
  return <>
    <div className="page-heading"><div><p className="eyebrow">{scripts ? "AUTOMATION" : "INFRASTRUCTURE"}</p><h1>{recipes ? "ComfyUI Recipes" : scripts ? "Scripts" : "Playbooks"} <span className="count">{shown.length}</span></h1><p>{scripts ? "Shell, Python and Deno sources in your workspace." : recipes ? "Pinned plugins and verified model downloads for existing RunPod SSH nodes." : "Editable multistep Playbooks with import and export."}</p></div><div className="inline-actions"><button disabled={!editable || busy} onClick={() => scripts ? create() : importFile()}><Icon name="Upload" />{scripts ? "Paste source" : "Import Playbook"}</button><button className="primary" disabled={!editable || busy} onClick={() => recipes ? setComfy(true) : create()}><Icon name="Plus" />{recipes ? "New recipe" : scripts ? "New script" : "New playbook"}</button></div></div>
    {error && !draft && <p className="form-error" role="alert">{error}</p>}

    {!scripts && !recipes && <div className="playbook-drop" role="region" aria-label="Import Playbook file" onDragOver={e => { if (e.dataTransfer.types.includes("Files")) e.preventDefault(); }} onDrop={e => {
      if (!e.dataTransfer.types.includes("Files")) return;
      e.preventDefault(); if (!editable || busy || draft) return;
      const files = e.dataTransfer.files;
      if (files.length !== 1) { setError("Drop one Playbook file at a time."); return; }
      const file = files[0];
      if (file.size > 256 * 1024) { setError("Playbooks must be at most 256 KiB."); return; }
      void run(async () => { const source = await file.text(); await api.notebookParse(source); create({ id: file.name.replace(/(?:\.steps)?\.(?:ya?ml|json)$/i, "").toLowerCase().replace(/[^a-z0-9_-]/g, "-").slice(0,64), kind: "notebook", source }); });
    }}>Drop a Multistep Playbook file here to import · .steps.yml, .yaml or .json</div>}
    <section className="panel"><div className="table-toolbar"><input data-surface-search aria-label={`Filter ${scripts ? "scripts" : "playbooks"}`} placeholder="Filter by name, ID or runtime…" value={query} onChange={e => setQuery(e.target.value)} /></div>
      {shown.length ? <div className="automation-list">{shown.filter(item => `${item.id} ${item.validation.name} ${labels[item.kind]}`.toLowerCase().includes(query.toLowerCase())).map(item => <article className="automation-row" key={`${item.kind}:${item.id}`}><div><strong title={item.validation.name}>{item.validation.name}</strong><p className="field-help">{item.id} · {labels[item.kind]} · {item.validation.valid ? `${item.validation.task_count || "Project"} ${item.validation.task_count ? "task(s)" : "link"}` : "Needs correction"}</p></div><div className="inline-actions">{item.kind === "notebook" && <button onClick={() => { void run(async () => { setExpanded(false); setNotebook(await api.automationRead({ context, id: item.id, kind: item.kind })); }); }}>Open Playbook {item.id}</button>}<button disabled={!editable || busy || !workspace.trusted || !item.validation.valid} title="Choose targets and run" aria-label={`Run ${item.id}`} onClick={() => { setRunCells([]); setRunMode(["playbook", "ansible"].includes(item.kind) ? "check" : "apply"); setRunItem(item); }}><Icon name="Play" />Run</button>{item.kind === "notebook" && <button disabled={busy} aria-label={`Export ${item.id}`} onClick={() => exportItem(item)}>Export</button>}<button disabled={!editable || busy} aria-label={`Edit ${item.id}`} onClick={() => edit(item)}><Icon name="Pencil" />Edit</button><button disabled={!editable || busy} aria-label={`Remove ${item.id}`} onClick={() => edit(item, true)}><Icon name="Trash2" />Remove</button></div></article>)}</div> : <div className="empty-state"><h3>{scripts ? "Create your first script" : "Create your first Playbook"}</h3><p>Start with an example below, or paste your own source.</p></div>}
    </section>
    {notebook && <section className={`panel playbook-view ${expanded ? "playbook-maximized" : ""}`}><div className="inline-actions"><button aria-pressed={expanded} onClick={() => setExpanded(value => !value)}>{expanded ? "Restore Playbook" : "Maximize Playbook"}</button><button onClick={() => { setNotebook(null); setExpanded(false); }}>Close Playbook</button></div><NotebookEditor source={notebook.source} disabled={!editable || busy || !workspace.trusted} reports={reports.filter(report => report.automation_id === notebook.id && report.kind === "notebook")} run={cells => { const item = items.find(value => value.id === notebook.id && value.kind === "notebook"); if (item) { setRunCells(cells); setRunMode("apply"); setRunItem(item); } }} /></section>}
    {!recipes && <section className="panel"><h2>Examples</h2><button disabled={!editable || busy} onClick={restoreExamples}>Restore example Playbooks</button>{restoreStatus && <p role="status">{restoreStatus}</p>}<div className="automation-examples">{offered.map(example => <article key={example.document.id}><h3>{example.label}</h3><p>{example.description}</p><button disabled={!editable || busy} onClick={() => create(example.document)}>Use example<Icon name="ArrowUpRight" /></button></article>)}</div></section>}
    <p className="field-help">{workspace.external_changed ? "Reload the workspace from Overview before editing." : "Saving keeps the exact source. Opening or validating a document does not execute it."} {scripts ? "Validation checks source bounds. Run executes on the selected targets and checks for the required interpreter." : "Each step runs independently on your selected targets. Saving does not execute steps."}</p>
    {comfy && <ComfyDialog dismiss={() => setComfy(false)} created={document => { setComfy(false); create({ ...document, id: document.id.startsWith("comfy-") ? document.id : `comfy-${document.id}` }); }} />}
    {runItem && <RunDialog workspace={workspace} item={runItem} initialCells={runCells} update={update} initialMode={runMode} dismiss={() => setRunItem(null)} started={() => { setRunItem(null); started(); }} busyChanged={busyChanged} />}
    {draft && <Dialog title="automation-title" className={`automation-dialog ${draft.document.kind === "notebook" ? "playbook-dialog" : ""} ${expanded ? "editor-expanded" : ""}`} busy={busy} dismiss={dismiss} save={saveDraft}>
      <h2 id="automation-title">{draft.remove ? "Remove" : draft.replace ? "Edit" : "Create"} {scripts ? "script" : "playbook"}</h2>
      {draft.remove ? <p>Remove {draft.document.id}? A recoverable file remains in the workspace folder.</p> : <>
        <div className="form-grid"><div><label htmlFor="automation-id">ID</label><input autoFocus data-dialog-focus id="automation-id" maxLength={64} aria-invalid={!idValid} aria-describedby="automation-id-help" value={draft.document.id} disabled={busy || draft.replace} onChange={e => change({ ...draft.document, id: e.target.value })} /><p id="automation-id-help" className={idValid ? "field-help" : "form-error"}>1–64 lowercase letters, digits, hyphens or underscores; start with a letter or digit. No spaces.</p>{!idValid && validId(suggestedId) && !draft.replace && <button type="button" disabled={busy} onClick={() => change({ ...draft.document, id: suggestedId })}>Use {suggestedId}</button>}</div>{recipes && <div><label htmlFor="automation-kind">Format</label><Select id="automation-kind" disabled={busy || draft.replace} value={draft.document.kind} onChange={e => {
          const kind = e.target.value as AutomationKind;
          setExpanded(kind === "notebook");
          const example = examples.find(item => item.document.kind === kind) || { document: { source: newNotebook } };
          change({ ...draft.document, kind, source: example.document.source });
        }}>{(scripts ? ["shell", "python", "deno"] : ["notebook", "playbook", "ansible"]).map(kind => <option value={kind} key={kind}>{labels[kind as AutomationKind]}</option>)}</Select></div>}</div>
        {!draft.replace && !recipes && <><label htmlFor="automation-example">Start from example</label><Select id="automation-example" value="" disabled={busy} onChange={e => change({ ...examples.find(item => item.document.id === e.target.value)!.document })}><option value="" disabled>Choose an example…</option>{offered.map(example => <option key={example.document.id} value={example.document.id}>{example.label}</option>)}</Select></>}
        {draft.document.kind === "notebook" && <button className="playbook-maximize" aria-pressed={expanded} onClick={() => setExpanded(value => !value)}>{expanded ? "Restore Playbook" : "Maximize Playbook"}</button>}
        {draft.document.kind === "notebook" ? <NotebookEditor source={draft.document.source} disabled={busy} change={source => change({ ...draft.document, source })} save={saveDraft} /> : <SourceEditor document={draft.document} disabled={busy} expanded={expanded} expand={() => setExpanded(value => !value)} change={source => change({ ...draft.document, source })} save={saveDraft} />}
        <p className="field-help">{draft.document.kind === "ansible" ? "Use an absolute project directory and paths relative to it. The linked project stays in place. Keep credentials in its existing private files." : "Do not put passwords, tokens or private keys in source. Use non-secret JSON inputs; credential storage for scripts is not connected yet."}</p>
      </>}
      {preview && <div className={preview.validation.valid ? "notice" : "form-error"} role="status">{preview.validation.valid ? `${draft.remove ? "Removal" : preview.validation.name} validated. ${preview.validation.task_count} task(s).` : preview.validation.issues.join(" ")}</div>}
      {error && <p className="form-error" role="alert">{error}</p>}
      {discard ? <div className="notice"><p>Discard this unsaved draft?</p><div className="inline-actions"><button autoFocus onClick={() => setDiscard(false)}>Keep editing</button><button onClick={() => { setDraft(null); setDiscard(false); }}>Discard draft</button></div></div> : <div className="modal-actions"><button disabled={busy} onClick={dismiss}>Cancel</button><button className="primary" disabled={busy || !idValid || !editable} onClick={saveDraft}>{draft.remove ? "Remove" : "Save"}</button></div>}
    </Dialog>}
  </>;
}
