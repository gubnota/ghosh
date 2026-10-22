import { useEffect, useRef, useState } from "react";
import type { editor } from "monaco-editor/editor/editor.api";
import { api, errorMessage } from "../../app/ipc";
import type { AutomationDocument } from "../../generated/contracts";

export function SourceEditor({ document, disabled, expanded, expand, change, save }: {
  document: AutomationDocument; disabled: boolean; expanded: boolean; expand?: () => void;
  change: (source: string) => void; save: () => void;
}) {
  const host = useRef<HTMLDivElement>(null), instance = useRef<editor.IStandaloneCodeEditor | null>(null);
  const current = useRef({ document, change, save }); current.current = { document, change, save };
  const [plain, setPlain] = useState(false), [ready, setReady] = useState(false), [status, setStatus] = useState("Loading offline editor…");
  const language = document.kind === "deno" ? "typescript" : document.kind === "playbook" || document.kind === "ansible" || document.kind === "notebook" ? "yaml" : document.kind;
  useEffect(() => {
    if (plain) return;
    let alive = true, dispose: (() => void) | undefined;
    setReady(false);
    void import("./editorRuntime").then(({ monaco }) => {
      if (!alive || !host.current) return;
      const model = monaco.editor.createModel(current.current.document.source, language);
      const view = monaco.editor.create(host.current, { model, theme: "ghosh-nightfly", automaticLayout: true, minimap: { enabled: false }, fontFamily: "'JetBrains Mono', monospace", fontSize: 13, lineHeight: 23, wordWrap: "on", scrollBeyondLastLine: false, tabSize: 2, insertSpaces: true, readOnly: disabled, quickSuggestions: true, suggestOnTriggerCharacters: true, fixedOverflowWidgets: false, ariaLabel: "Source code editor", padding: { top: 12, bottom: 12 } });
      instance.current = view;
      const listener = model.onDidChangeContent(() => current.current.change(model.getValue()));
      view.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => current.current.save());
      dispose = () => { listener.dispose(); view.dispose(); model.dispose(); instance.current = null; };
      setReady(true);
    }).catch(() => { if (alive) { setPlain(true); setStatus("Code editor unavailable. Plain text editing remains available."); } });
    return () => { alive = false; dispose?.(); };
  }, [plain, language]);
  useEffect(() => { const model = instance.current?.getModel(); if (model && model.getValue() !== document.source) model.setValue(document.source); }, [document.source]);
  useEffect(() => { instance.current?.updateOptions({ readOnly: disabled }); }, [disabled, ready]);
  useEffect(() => {
    if (!ready && !plain) return;
    if (document.kind === "deno") { setStatus("Bundled TypeScript checks and completion · basic Deno API types; remote imports are not resolved."); return; }
    let alive = true;
    const timer = window.setTimeout(() => {
      void api.automationLint(document).then(async result => {
        if (!alive) return;
        setStatus(`${result.engine} · ${result.diagnostics.length ? `${result.diagnostics.length} issue(s): ${result.diagnostics[0].message}` : "No syntax issues"}`);
        if (plain) return;
        const { monaco } = await import("./editorRuntime");
        const model = instance.current?.getModel();
        if (!alive || !model) return;
        monaco.editor.setModelMarkers(model, "ghosh", result.diagnostics.map(issue => {
          const line = Math.max(1, Math.min(model.getLineCount(), issue.line));
          return { startLineNumber: line, endLineNumber: line, startColumn: Math.max(1, issue.column), endColumn: model.getLineMaxColumn(line), message: issue.message, severity: monaco.MarkerSeverity.Error };
        }));
      }).catch(e => { if (alive) setStatus(`Diagnostics unavailable: ${errorMessage(e)}`); });
    }, 450);
    return () => { alive = false; window.clearTimeout(timer); };
  }, [document.source, document.kind, document.id, ready, plain]);
  return <section className="source-workbench" aria-label="Step code editor">
    <div className="editor-toolbar"><strong>{document.kind === "ansible" ? "Project link · YAML" : "Source"}</strong><div className="inline-actions"><button type="button" onClick={() => setPlain(value => !value)}>{plain ? "Code editor" : "Plain text"}</button>{expand && <button type="button" aria-pressed={expanded} onClick={expand}>{expanded ? "Restore editor" : "Maximize editor"}</button>}</div></div>
    {plain ? <textarea aria-label="Source" className="source-editor" spellCheck={false} value={document.source} disabled={disabled} onChange={event => change(event.target.value)} /> : <div ref={host} className="monaco-host" />}
    <p className="editor-diagnostics" role="status">{status}</p><p className="field-help">⌘S Save · Ctrl+Space Suggestions · Ctrl+M Switch Tab between indent and keyboard navigation · Esc Restore expanded editor</p>
  </section>;
}
