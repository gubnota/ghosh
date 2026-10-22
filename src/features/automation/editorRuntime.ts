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
const snippets: Record<string, [string, string][]> = {
  shell: [
    ["function", '${1:name}() {\n  ${2:echo ready}\n}'],
    ["if", 'if ${1:command}; then\n  ${2:echo ok}\nfi'],
    ["for", 'for item in ${1:items}; do\n  echo "$item"\ndone'],
    ["while", 'while ${1:command}; do\n  ${2:echo ready}\ndone'],
    ["case", 'case "${1:value}" in\n  ${2:pattern}) ${3:echo match} ;;\n  *) echo other ;;\nesac'],
    ["set", "set -eu"],
  ],
  python: [
    ["def", "def ${1:name}(${2:args}):\n    ${3:pass}"],
    ["class", "class ${1:Name}:\n    def __init__(self, ${2:args}):\n        ${3:pass}"],
    ["if", "if ${1:condition}:\n    ${2:pass}"],
    ["for", "for ${1:item} in ${2:items}:\n    ${3:pass}"],
    ["while", "while ${1:condition}:\n    ${2:pass}"],
    ["try", "try:\n    ${1:pass}\nexcept ${2:Exception} as error:\n    ${3:print(error)}"],
    ["with", 'with open(${1:path}) as ${2:file}:\n    ${3:print(file.read())}'],
    ["main", 'def main():\n    ${1:pass}\n\nif __name__ == "__main__":\n    main()'],
    ["print", "print(${1:value})"],
  ],
  typescript: [
    ["function", "function ${1:name}(${2:args}): ${3:void} {\n  ${4:}\n}"],
    ["async function", "async function ${1:name}(${2:args}): Promise<${3:void}> {\n  ${4:}\n}"],
    ["arrow", "const ${1:name} = (${2:args}) => {\n  ${3:}\n};"],
    ["for", "for (const ${1:item} of ${2:items}) {\n  ${3:}\n}"],
    ["if", "if (${1:condition}) {\n  ${2:}\n}"],
    ["try", "try {\n  ${1:}\n} catch (error) {\n  console.error(error);\n}"],
    ["class", "class ${1:Name} {\n  constructor(${2:}) {\n    ${3:}\n  }\n}"],
  ],
  yaml: [["task", "- name: ${1:Step}\n  check:\n    command: ${2:test -f /tmp/example || exit 10}\n  shell:\n    command: |\n      ${3:echo ready}\n  timeout_s: 30"], ["shell", "shell:\n  command: |\n    ${1:echo ready}"]],
};
const keywords: Record<string, string[]> = {
  shell: "if then else elif fi for in do done while until case esac function return break continue export readonly local echo printf test cd mkdir cp mv rm cat grep sed awk exit trap unset read command".split(" "),
  python: "and as assert async await break class continue def del elif else except False finally for from global if import in is lambda None nonlocal not or pass raise return True try while with yield len range enumerate zip list dict set tuple str int float bool open print input isinstance super sorted sum min max".split(" "),
};
for (const language of Object.keys(snippets)) monaco.languages.registerCompletionItemProvider(language, {
  provideCompletionItems(model, position) {
    const word = model.getWordUntilPosition(position), range = { startLineNumber: position.lineNumber, endLineNumber: position.lineNumber, startColumn: word.startColumn, endColumn: word.endColumn };
    return { suggestions: [
      ...snippets[language].map(([label, insertText]) => ({ label, insertText, range, kind: monaco.languages.CompletionItemKind.Snippet, insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, detail: "Insert construction · Tab through fields", sortText: `0-${label}` })),
      ...(keywords[language] || []).filter(label => !snippets[language].some(([name]) => name === label)).map(label => ({ label, insertText: label, range, kind: monaco.languages.CompletionItemKind.Keyword, detail: "Keyword / common builtin", sortText: `1-${label}` })),
    ] };
  },
});
export { monaco };
