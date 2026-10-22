import { useEffect, useRef, useState } from "react";
import { commands } from "../app/commands";
import { Shortcut } from "./Shortcut";

export const shortcuts = [
  { label: "Command popup", shortcut: "⌘K / ⌘⇧P", context: "Application" },
  { label: "Show shortcuts (hold)", shortcut: "⌘⌥", context: "Application; release or Escape to hide" },
  { label: "Save current document", shortcut: "⌘S", context: "Editor or workspace form" },
  { label: "Search current surface", shortcut: "⌘F", context: "Active searchable view" },
  { label: "Close terminal session / active tab", shortcut: "⌘W", context: "Terminal confirms termination; otherwise closes tab" },
  { label: "Previous / next terminal session", shortcut: "⌘⇧[ / ⌘⇧]", context: "Terminal; focuses the selected shell" },
  { label: "Switch to tab 1–9", shortcut: "⌘1–9", context: "Application" },
  { label: "Switch application tabs", shortcut: "⌘[ / ⌘] · Ctrl+Tab / Ctrl+Shift+Tab", context: "Application" },
  { label: "Code suggestions", shortcut: "Ctrl+Space", context: "Code editor" },
  { label: "Indent / move focus with Tab", shortcut: "Ctrl+M", context: "Code editor" },
  { label: "Restore expanded editor / dismiss dialog", shortcut: "Esc", context: "Expanded step, editor or dialog" },
  { label: "Navigate controls", shortcut: "Tab / Shift+Tab", context: "Application" },
  { label: "Select / confirm", shortcut: "↑ ↓ · Enter / Space", context: "Menus and buttons" },
  { label: "Find previous / next match", shortcut: "Shift+Enter / Enter", context: "Terminal search field" },
  { label: "Interrupt shell command", shortcut: "Ctrl+C", context: "Focused terminal; shell control keys pass through" },
  ...commands.filter(command => command.shortcut).map(command => ({ label: command.label, shortcut: command.shortcut!, context: command.group })),
];
export function ShortcutTable({ search = "" }: { search?: string }) {
  return <div className="table-scroll"><table><thead><tr><th>Action</th><th>Shortcut</th><th>Context</th></tr></thead><tbody>{shortcuts.filter(item => `${item.label} ${item.context}`.toLowerCase().includes(search.toLowerCase())).map(item => <tr key={item.label}><td>{item.label}</td><td><Shortcut keys={item.shortcut} /></td><td>{item.context}</td></tr>)}</tbody></table></div>;
}
export function ShortcutOverlay() {
  const [held, setHeld] = useState(false);
  const overlay = useRef<HTMLDivElement>(null), dismissed = useRef(false);
  useEffect(() => {
    const keys = (event: KeyboardEvent) => {
      if (event.key === "Escape") { if (overlay.current?.matches(":popover-open")) { event.preventDefault(); event.stopImmediatePropagation(); } dismissed.current = true; setHeld(false); return; }
      const pressed = event.metaKey && event.altKey;
      if (!pressed) dismissed.current = false;
      if (!event.isComposing) setHeld(pressed && !dismissed.current);
    };
    const blur = () => { setHeld(false); dismissed.current = false; };
    window.addEventListener("keydown", keys, true); window.addEventListener("keyup", keys, true); window.addEventListener("blur", blur);
    return () => { window.removeEventListener("keydown", keys, true); window.removeEventListener("keyup", keys, true); window.removeEventListener("blur", blur); };
  }, []);
  useEffect(() => { if (held) overlay.current?.showPopover(); else overlay.current?.hidePopover(); }, [held]);
  return <div ref={overlay} popover="manual" className="shortcut-overlay" role="region" aria-label="Keyboard shortcut overlay"><h2>Keyboard shortcuts</h2><p>Release Command or Option to hide. Open “Keyboard shortcuts” in Commands for a searchable view.</p><ShortcutTable /></div>;
}
