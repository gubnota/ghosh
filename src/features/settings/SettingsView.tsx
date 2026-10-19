import { APP_VERSION } from "../../app/version";
import { Shortcut } from "../../components/Shortcut";
import { Icon } from "../../components/Icon";
import { Select } from "../../components/Select";
import { useState } from "react";
import { commands } from "../../app/commands";

export type Appearance = { fontSize: number; density: "comfortable" | "compact"; zoom: number };
const defaults: Appearance = { fontSize: 14, density: "comfortable", zoom: 100 };
export function loadAppearance(): Appearance {
  try {
    const saved = JSON.parse(localStorage.getItem("ghosh.appearance.v1") || "null");
    return { fontSize: [13, 14, 15, 16, 18].includes(saved?.fontSize) ? saved.fontSize : 14, density: saved?.density === "compact" ? "compact" : "comfortable", zoom: [100, 125, 150, 200].includes(saved?.zoom) ? saved.zoom : 100 };
  } catch { return defaults; }
}
export function saveAppearance(value: Appearance): boolean { try { localStorage.setItem("ghosh.appearance.v1", JSON.stringify(value)); return true; } catch { return false; } }
export function SettingsView({ appearance, change, keyboard = false }: { appearance: Appearance; change: (value: Appearance) => void; keyboard?: boolean }) {
  const [category, setCategory] = useState(keyboard ? "Keyboard shortcuts" : "Appearance");
  const [search, setSearch] = useState("");
  return <><div className="page-heading"><div><p className="eyebrow">APPLICATION</p><h1>Settings</h1><p>Preferences stay on this machine.</p></div></div>
    <div className="settings-layout"><nav aria-label="Settings categories">{["Appearance", "Keyboard shortcuts", "About"].map(item => <button key={item} aria-current={category === item ? "page" : undefined} className={category === item ? "selected" : "quiet"} onClick={() => setCategory(item)}><Icon name={item === "Appearance" ? "Settings" : item === "Keyboard shortcuts" ? "Keyboard" : "Info"} />{item}</button>)}</nav>
    <section className="panel"><h2>{category}</h2>{category === "Appearance" ? <><p>Nightfly Default · local monospace fonts · no font downloads.</p><label htmlFor="font-size">Font size</label><Select id="font-size" value={appearance.fontSize} onChange={e => change({ ...appearance, fontSize: Number(e.target.value) })}>{[13, 14, 15, 16, 18].map(size => <option key={size} value={size}>{size}px</option>)}</Select><label htmlFor="density">Row spacing</label><Select id="density" value={appearance.density} onChange={e => change({ ...appearance, density: e.target.value as Appearance["density"] })}><option value="comfortable">Comfortable</option><option value="compact">Compact</option></Select><label htmlFor="ui-zoom">Interface zoom</label><Select id="ui-zoom" value={appearance.zoom} onChange={e => change({ ...appearance, zoom: Number(e.target.value) })}>{[100, 125, 150, 200].map(zoom => <option value={zoom} key={zoom}>{zoom}%</option>)}</Select><div className="font-preview"><span className="accent">ghosh &gt;</span> Ready. 0O 1lI {'{} [] ()'}<br /><span className="muted">Readable text, clear focus, ordinary controls.</span></div><button onClick={() => change(defaults)}>Reset appearance</button></> : category === "Keyboard shortcuts" ? <><p>Cmd+K or Cmd+Shift+P opens commands. Every control supports Tab and Shift+Tab.</p><label htmlFor="shortcut-search">Find a shortcut</label><input id="shortcut-search" value={search} onChange={e => setSearch(e.target.value)} placeholder="Search actions…" /><div className="table-scroll"><table><thead><tr><th>Action</th><th>Shortcut</th></tr></thead><tbody>{[
      { label: "Command popup", shortcut: "⌘K / ⌘⇧P" }, { label: "Save current document", shortcut: "⌘S" }, { label: "Search current surface", shortcut: "⌘F" }, { label: "Close active tab", shortcut: "⌘W" }, { label: "Switch to tab 1–9", shortcut: "⌘1–9" }, { label: "Switch tabs", shortcut: "⌘[ / ⌘] · Ctrl+Tab / Ctrl+Shift+Tab" },
      ...commands.filter(command => command.shortcut),
    ].filter(item => item.label.toLowerCase().includes(search.toLowerCase())).map(item => <tr key={item.label}><td>{item.label}</td><td><Shortcut keys={item.shortcut!} /></td></tr>)}</tbody></table></div><p className="field-help">Commands show a reason when their service is unavailable. Shortcut remapping will arrive with the remaining settings.</p></> : <><p>Ghosh {APP_VERSION} · desktop workspace foundation</p><p>MIT OR Apache-2.0. Files stay local. Opening a workspace starts no network requests or scripts.</p><p>Operational services are being delivered sequentially. Managed SSH terminals are available. Playbook execution is pending.</p></>}</section></div>
  </>;
}
