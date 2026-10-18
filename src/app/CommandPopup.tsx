import { useId, useState } from "react";
import { Dialog } from "../components/Dialog";
import { commandReason, searchCommands, type Command } from "./commands";

export function CommandPopup({ dismiss, execute, context }: {
  dismiss: () => void; execute: (command: Command) => void; context: { desktop: boolean; workspace: boolean; busy: boolean; editable?: boolean; trusted?: boolean };
}) {
  const id = useId();
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState(0);
  const matches = searchCommands(query);
  const active = matches[Math.min(selected, Math.max(matches.length - 1, 0))];
  const reason = active ? commandReason(active, context) : null;
  const dispatch = (command: Command) => { if (!commandReason(command, context)) execute(command); };
  return <Dialog title={`${id}-title`} dismiss={dismiss} className="command-popup">
    <h2 id={`${id}-title`} className="sr-only">Command popup</h2>
    <div className="command-input"><span aria-hidden="true">ghosh &gt;</span><input autoFocus data-dialog-focus aria-label="Search commands" placeholder="Type a command or search…" value={query}
      role="combobox" aria-expanded="true" aria-controls={`${id}-list`} aria-autocomplete="list" aria-activedescendant={active ? `${id}-${active.id}` : undefined}
      onChange={event => { setQuery(event.target.value); setSelected(0); }} onKeyDown={event => {
        if (event.key === "ArrowDown" || event.key === "ArrowUp") { event.preventDefault(); setSelected(index => matches.length ? (index + (event.key === "ArrowDown" ? 1 : -1) + matches.length) % matches.length : 0); }
        if (event.key === "Enter") { event.preventDefault(); if (active) dispatch(active); }
      }} /><kbd>esc</kbd></div>
    <div id={`${id}-list`} role="listbox" aria-label="Commands" className="command-list">
      {matches.map((command, index) => <div key={command.id} id={`${id}-${command.id}`} role="option" aria-selected={active?.id === command.id} aria-disabled={!!commandReason(command, context)}
        className={`command-option ${active?.id === command.id ? "active" : ""}`} onMouseMove={() => setSelected(index)} onClick={() => dispatch(command)} ref={element => { if (active?.id === command.id) element?.scrollIntoView({ block: "nearest" }); }}>
        <span><strong>{command.label}</strong><small>{command.syntax}</small></span><span className="command-group">{command.shortcut || command.group}</span>
      </div>)}
      {!matches.length && <p className="command-empty">No matching command. Try “workspace”, “terminal” or “settings”. Arguments will be available with their service.</p>}
    </div>
    <div className="command-footer" role="status">{reason || (active ? "↑ ↓ choose · Enter open · Esc close" : "No command selected")}</div>
  </Dialog>;
}
