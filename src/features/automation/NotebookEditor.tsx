import { useEffect, useState, useRef } from "react";
import { api, errorMessage } from "../../app/ipc";
import type { Notebook, NotebookCell, RunReport } from "../../generated/contracts";
import { Select } from "../../components/Select";
import { SourceEditor } from "./SourceEditor";

export const newNotebook = JSON.stringify({ schema: 1, name: "Multistep Playbook", cells: [{ id: "step-1", title: "First step", runtime: "shell", code: "printf 'Hello from step 1\\n'", timeout_s: 30, success_check: null }] }, null, 2);
export function NotebookEditor({ source, change, disabled, run, reports = [], save }: {
  source: string; change?: (source: string) => void; disabled: boolean;
  run?: (cells: string[]) => void; reports?: RunReport[]; save?: () => void;
}) {
  const [book, setBook] = useState<Notebook | null>(null), [error, setError] = useState("");
  const [expanded, setExpanded] = useState("");
  const dragged = useRef<string | null>(null);
  const [announcement, setAnnouncement] = useState("");
  useEffect(() => { let alive = true; void api.notebookParse(source).then(value => { if (alive) { setBook(value); setError(""); } }).catch(e => { if (alive) setError(errorMessage(e)); }); return () => { alive = false; }; }, [source]);
  const write = (next: Notebook) => { setBook(next); change?.(JSON.stringify(next, null, 2)); };
  const patch = (id: string, values: Partial<NotebookCell>) => { if (book) write({ ...book, cells: book.cells.map(cell => cell.id === id ? { ...cell, ...values } : cell) }); };
  const reorder = (id: string, target: string) => {
    if (!book || disabled || !change || id === target) return;
    const from = book.cells.findIndex(cell => cell.id === id), to = book.cells.findIndex(cell => cell.id === target);
    if (from < 0 || to < 0) return;
    const cells = [...book.cells]; const [step] = cells.splice(from, 1); cells.splice(to, 0, step);
    write({ ...book, cells }); setAnnouncement(`${step.title} moved to step ${to + 1}`);
    requestAnimationFrame(() => document.getElementById(`drag-${id}`)?.focus());
  };
  const move = (index: number, offset: number) => { if (book) reorder(book.cells[index].id, book.cells[index + offset].id); };
  return <section className="notebook" aria-label="Playbook steps">
    <p className="sr-only" role="status">{announcement}</p>
    <p className="field-help">Each step starts a fresh process. Files persist on the target; variables and directory changes do not carry between steps. Running a step again repeats its code.</p>
    {error && <p className="form-error" role="alert">{error}</p>}
    {!book && !error && <p>Loading steps…</p>}
    {book && <>
      {change ? <label>Playbook name<input value={book.name} disabled={disabled} onChange={e => write({ ...book, name: e.target.value })} /></label> : <div className="section-heading"><h2>{book.name}</h2><button disabled={disabled} onClick={() => run?.([])}>Run all steps</button></div>}
      {book.cells.map((cell, index) => {
        const report = reports.find(value => value.results.some(result => result.name.startsWith(`${cell.id} · `)) || value.progress?.step.startsWith(`${cell.id} · `));
        const results = report?.results.filter(result => result.name.startsWith(`${cell.id} · `)) || [];
        const live = report?.state === "running" && report.progress?.step.startsWith(`${cell.id} · `) ? report.progress : null;
        return <article className={`notebook-cell ${expanded === cell.id ? "step-maximized" : ""}`} key={cell.id} aria-label={`Step ${index + 1}: ${cell.title}`}
          onKeyDown={e => { if (expanded === cell.id && e.key === "Escape") { e.preventDefault(); e.stopPropagation(); setExpanded(""); } }}
          onDragOver={e => { if (change && !disabled && dragged.current) { e.preventDefault(); e.dataTransfer.dropEffect = "move"; } }}
          onDrop={e => { e.preventDefault(); const id = dragged.current; dragged.current = null; if (id) reorder(id, cell.id); }}>
          <div className="section-heading">{change && <button id={`drag-${cell.id}`} className="step-drag" aria-label={`Drag step ${index + 1}: ${cell.title}`} title="Drag to reorder; use Move up or Move down with the keyboard" draggable={!disabled} disabled={disabled} onDragStart={e => { dragged.current = cell.id; e.dataTransfer.effectAllowed = "move"; e.dataTransfer.setData("text/plain", cell.id); }} onDragEnd={() => { dragged.current = null; }}>⠿</button>}<strong>{index + 1} · {cell.title}</strong><button aria-pressed={expanded === cell.id} onClick={() => setExpanded(expanded === cell.id ? "" : cell.id)}>{expanded === cell.id ? "Restore step" : "Expand step"}</button><span>{live ? "Running" : results.at(-1)?.state || "Ready"}</span></div>
          {change ? <><label>Step title<input value={cell.title} disabled={disabled} onChange={e => patch(cell.id, { title: e.target.value })} /></label><div className="step-settings"><label>Runtime<Select aria-label="Runtime" value={cell.runtime} disabled={disabled} onChange={e => patch(cell.id, { runtime: e.target.value as NotebookCell["runtime"] })}><option value="shell">Shell</option><option value="python">Python</option><option value="deno">Deno</option></Select></label><label>Timeout (seconds; 0 = none)<input type="number" min={0} max={3600} value={Math.max(0, cell.timeout_s)} disabled={disabled} onChange={e => patch(cell.id, { timeout_s: Number(e.target.value) })} /></label></div></> : <p className="field-help">{cell.runtime} · {cell.timeout_s <= 0 ? "No timeout" : `${cell.timeout_s}s`} · {cell.id}</p>}
          <SourceEditor document={{ id: cell.id, kind: cell.runtime, source: cell.code }} disabled={disabled || !change} expanded={false} change={code => patch(cell.id, { code })} save={() => save?.()} />
          {change ? <><label>Success check · optional Shell command<textarea rows={2} disabled={disabled} value={cell.success_check || ""} onChange={e => patch(cell.id, { success_check: e.target.value || null })} /></label><div className="inline-actions"><button disabled={disabled || index === 0} onClick={() => move(index, -1)}>Move up</button><button disabled={disabled || index === book.cells.length - 1} onClick={() => move(index, 1)}>Move down</button><button disabled={disabled || book.cells.length === 1} onClick={() => write({ ...book, cells: book.cells.filter(value => value.id !== cell.id) })}>Remove step</button></div></> : <button disabled={disabled} onClick={() => run?.([cell.id])}>Run step {index + 1}</button>}
          {results.map((result, i) => <div key={i}><p className="field-help">{result.node_id} · {result.phase} · {result.state} · exit {result.exit_code ?? "unknown"}</p><pre tabIndex={0}>{result.output || "No output"}</pre></div>)}
          {live && <pre aria-label={`Step ${index + 1} live output`} tabIndex={0}>{live.output || "Waiting for output…"}</pre>}
        </article>;
      })}
      {change && <button disabled={disabled || book.cells.length >= 256} onClick={() => write({ ...book, cells: [...book.cells, { id: `cell-${crypto.randomUUID().slice(0, 8)}`, title: "New step", runtime: "shell", code: "echo ready", timeout_s: 30, success_check: null }] })}>Add step</button>}
    </>}
    {error && change && <label>Playbook source<textarea rows={12} value={source} onChange={e => change(e.target.value)} disabled={disabled} /></label>}
  </section>;
}
