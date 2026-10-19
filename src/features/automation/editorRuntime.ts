import * as monaco from "monaco-editor/editor/editor.api";
import EditorWorker from "monaco-editor/editor/editor.worker?worker";
import TypeScriptWorker from "monaco-editor/languages/features/typescript/ts.worker?worker";
import { typescriptDefaults, ScriptTarget, ModuleKind } from "monaco-editor/languages/features/typescript/register";
import "monaco-editor/languages/definitions/shell/register";
import "monaco-editor/languages/definitions/python/register";
import "monaco-editor/languages/definitions/yaml/register";
import "monaco-editor/languages/definitions/typescript/register";
import "monaco-editor/languages/definitions/javascript/register";
import "monaco-editor/editor/contrib/suggest/browser/suggestController";
import "monaco-editor/editor/contrib/snippet/browser/snippetController2";
import "monaco-editor/editor/contrib/hover/browser/hoverContribution";
import "monaco-editor/editor/contrib/find/browser/findController";
import "monaco-editor/editor/contrib/gotoError/browser/markerSelectionStatus";
import "monaco-editor/editor/contrib/wordOperations/browser/wordOperations";
import "monaco-editor/editor/contrib/linesOperations/browser/linesOperations";
import "monaco-editor/editor/contrib/toggleTabFocusMode/browser/toggleTabFocusMode";
import "monaco-editor/editor/contrib/tokenization/browser/tokenization";
import "monaco-editor/editor/browser/coreCommands";
import "../../../node_modules/monaco-editor/esm/vs/base/browser/ui/codicons/codicon/codicon.css";

(globalThis as typeof globalThis & { MonacoEnvironment: unknown }).MonacoEnvironment = {
  getWorker: (_: string, label: string) => label === "typescript" || label === "javascript" ? new TypeScriptWorker() : new EditorWorker(),
};
typescriptDefaults.setCompilerOptions({ target: ScriptTarget.ESNext, module: ModuleKind.ESNext, strict: true, noEmit: true, allowNonTsExtensions: true });
typescriptDefaults.addExtraLib("declare namespace Deno { function hostname(): string; const args: string[]; const env: { get(name: string): string | undefined }; function readTextFile(path: string): Promise<string>; function writeTextFile(path: string, data: string): Promise<void>; }", "file:///ghosh/deno.d.ts");
monaco.editor.defineTheme("ghosh-nightfly", { base: "vs-dark", inherit: true, rules: [
  { token: "comment", foreground: "637777", fontStyle: "italic" }, { token: "keyword", foreground: "C792EA" },
  { token: "string", foreground: "ECC48D" }, { token: "number", foreground: "F78C6C" },
], colors: { "editor.background": "#011627", "editor.foreground": "#C3CCDC", "editorLineNumber.foreground": "#637777", "editor.selectionBackground": "#1D3B53", "editor.lineHighlightBackground": "#082333", "editorWidget.background": "#0B253A", "editorSuggestWidget.background": "#0B253A" } });
for (const language of ["shell", "python", "yaml"]) monaco.languages.registerCompletionItemProvider(language, {
  provideCompletionItems(model, position) {
    const word = model.getWordUntilPosition(position), range = { startLineNumber: position.lineNumber, endLineNumber: position.lineNumber, startColumn: word.startColumn, endColumn: word.endColumn };
    const snippets = language === "shell" ? [["if", "if ${1:command}; then\n  ${2:echo ok}\nfi"], ["for", "for item in ${1:items}; do\n  echo \"$item\"\ndone"], ["set", "set -eu"]] : language === "python" ? [["def", "def ${1:name}(${2:args}):\n    ${3:pass}"], ["if", "if ${1:condition}:\n    ${2:pass}"], ["print", "print(${1:value})"]] : [["task", "- name: ${1:Step}\n  check:\n    command: ${2:test -f /tmp/example || exit 10}\n  shell:\n    command: |\n      ${3:echo ready}\n  timeout_s: 30"], ["shell", "shell:\n  command: |\n    ${1:echo ready}"]];
    return { suggestions: snippets.map(([label, insertText]) => ({ label, insertText, range, kind: monaco.languages.CompletionItemKind.Snippet, insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, detail: "Ghosh offline snippet" })) };
  },
});
export { monaco };
