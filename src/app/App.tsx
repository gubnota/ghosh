import { ShortcutOverlay } from "../components/ShortcutOverlay";
import { Icon } from "../components/Icon";
import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api, desktop, errorMessage } from "./ipc";
import type { RecentWorkspace, WorkspaceSnapshot, TerminalSession } from "../generated/contracts";
import { RecentWorkspaces } from "../features/workspace/RecentWorkspaces";
import { WorkspaceView } from "../features/workspace/WorkspaceView";
import { Dialog } from "../components/Dialog";
import { AppShell } from "./AppShell";
import { CommandPopup } from "./CommandPopup";
import { destinations, type Command, type Destination } from "./commands";
import { closeTab, openTab, tabId, type AppTab } from "./tabs";
import { loadAppearance, saveAppearance, SettingsView, type Appearance } from "../features/settings/SettingsView";
import { useOperations } from "./useOperations";
import { JobsView } from "../features/operations/JobsView";
import { ApprovalsView } from "../features/operations/ApprovalsView";
import { NodesView } from "../features/nodes/NodesView";
import { KeysView, type KeyAction } from "../features/keys/KeysView";

import { MonitorView } from "../features/monitor/MonitorView";
import { RunsView } from "../features/automation/RunsView";
import { AutomationView, type AutomationAction } from "../features/automation/AutomationView";
import { TerminalView } from "../features/terminal/TerminalView";
import { shortcutKey } from "./keyboard";

export function App() {
  const [terminalCount, setTerminalCount] = useState(0);
  const [terminalModal, setTerminalModal] = useState(false);
  const [incomingTerminal, setIncomingTerminal] = useState<TerminalSession | null>(null);
  const [terminalRequest, setTerminalRequest] = useState(0);
  const terminalSequence = useRef(0);
  const operations = useOperations();
  const [runStarting, setRunStarting] = useState(false);
  const runActive = runStarting || !!operations.snapshot?.jobs.jobs.some(job => job.kind === "automation_run" && ["running", "cancelling"].includes(job.state));
  const blockingOperation = !!operations.snapshot?.jobs.jobs.some(job => job.kind !== "automation_run" && ["running", "cancelling"].includes(job.state));
  const automationSequence = useRef(0);
  const [automationRequest, setAutomationRequest] = useState<{ action: AutomationAction; scripts: boolean; sequence: number } | null>(null);
  const [automationModal, setAutomationModal] = useState(false);
  const [automationBusy, setAutomationBusy] = useState(false);
  const [inventoryModal, setInventoryModal] = useState(false);
  const [inventoryBusy, setInventoryBusy] = useState(false);
  const [keysModal, setKeysModal] = useState(false);
  const [keysBusy, setKeysBusy] = useState(false);
  const keySequence = useRef(0);
  const [keyRequest, setKeyRequest] = useState<{ action: KeyAction; sequence: number } | null>(null);
  const nodeSequence = useRef(0);
  const [nodeRequest, setNodeRequest] = useState<{ action: "create" | "edit" | "test" | "terminal"; sequence: number } | null>(null);
  const [approvalModal, setApprovalModal] = useState(false);
  const [workspace, setWorkspace] = useState<WorkspaceSnapshot | null>(null);
  const [startupBusy, setStartupBusy] = useState(desktop);
  const [recentOpen, setRecentOpen] = useState(false);
  const [recent, setRecent] = useState<RecentWorkspace[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dirty, setDirty] = useState(false);
  const [create, setCreate] = useState(false);
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [location, setLocation] = useState("");
  const [openPath, setOpenPath] = useState("");
  const [confirmation, setConfirmation] = useState<{ title: string; action: () => void } | null>(null);
  const [tabs, setTabs] = useState<AppTab[]>([{ id: "app:overview", destination: "overview", title: "Welcome" }]);
  const [active, setActive] = useState("app:overview");
  const [commandOpen, setCommandOpen] = useState(false);
  const [keyboardHelp, setKeyboardHelp] = useState(false);
  const [appearance, setAppearance] = useState(loadAppearance);
  const [preferenceNotice, setPreferenceNotice] = useState("");
  const current = useRef({ dirty, terminalCount, workspace, runActive, busy: startupBusy || busy || blockingOperation || inventoryBusy || keysBusy || automationBusy, confirming: startupBusy || recentOpen || !!confirmation || create || open || commandOpen || keyboardHelp || approvalModal || inventoryModal || keysModal || terminalModal || automationModal }); current.current = { dirty, terminalCount, workspace, runActive, busy: startupBusy || busy || blockingOperation || inventoryBusy || keysBusy || automationBusy, confirming: startupBusy || recentOpen || !!confirmation || create || open || commandOpen || keyboardHelp || approvalModal || inventoryModal || keysModal || terminalModal || automationModal };
  const queuedOpen = useRef(false);
  const allowClose = useRef(false);
  const confirm = (title: string, action: () => void) => setConfirmation({ title, action });
  const guard = (action: () => void) => runActive ? setError("Stop the active run in Runs and wait for its result before switching workspace.") : current.current.terminalCount ? confirm("Terminate terminal sessions before switching workspace?", action) : current.current.dirty ? confirm("Discard unsaved changes before continuing?", action) : action();
  const run = async (action: () => Promise<void>) => { if (current.current.busy) return; current.current.busy = true; setBusy(true); setError(null); try { await action(); } catch (e) { setError(errorMessage(e)); } finally { current.current.busy = false; setBusy(false); } };
  const accept = (s: WorkspaceSnapshot) => { setWorkspace(s); setDirty(false); void api.recent().then(setRecent).catch(() => {}); };
  const openWorkspace = (path: string) => guard(() => { void run(async () => { accept(await api.open(path)); setOpen(false); setRecentOpen(false); }); });
  const receiveOpen = async () => { if (current.current.busy || current.current.confirming) { queuedOpen.current = true; return; } queuedOpen.current = false; const path = await api.pending(); if (path) { queuedOpen.current = true; openWorkspace(path); } };
  useEffect(() => { if (desktop && !busy && !confirmation && queuedOpen.current) { void receiveOpen().catch(e => setError(errorMessage(e))); } }, [busy, confirmation, create, open, commandOpen, keyboardHelp, approvalModal, inventoryModal, inventoryBusy, keysModal, keysBusy, terminalModal, automationModal, automationBusy, operations.mutating, recentOpen, startupBusy]);
  useEffect(() => {
    if (!desktop) return;
    let alive = true; const disposers: (() => void)[] = [];
    const register = async (registration: Promise<() => void>) => { const stop = await registration; if (alive) disposers.push(stop); else stop(); };
    void (async () => {
      try {
        await register(api.onFiles(event => {
          if (current.current.workspace?.id !== event.workspace_id) return;
          void api.snapshot().then(snapshot => { if (alive && snapshot && snapshot.id === current.current.workspace?.id) setWorkspace(snapshot); }).catch(e => setError(errorMessage(e)));
        }));
        await register(api.onOpen(() => { void receiveOpen().catch(e => setError(errorMessage(e))); }));
        await register(getCurrentWindow().onCloseRequested(event => {
          if (allowClose.current) return;
          if (current.current.busy || current.current.dirty || current.current.terminalCount || current.current.confirming) {
            event.preventDefault();
            if (current.current.busy) { setError("Wait for the current operation to finish before closing Ghosh."); return; }
            if (current.current.confirming) { setError("Close the open dialog before closing Ghosh."); return; }
            confirm(current.current.terminalCount ? "Terminate terminal sessions and close Ghosh?" : "Discard unsaved changes and close Ghosh?", () => { allowClose.current = true; void getCurrentWindow().close(); });
          }
        }));
        const snapshot = await api.startup(); if (!alive) return; setWorkspace(snapshot);
      } catch (e) { if (alive) setError(`Could not restore the workspace. Open another workspace or remove the missing entry from history. ${errorMessage(e)}`); }
      finally {
        if (alive) {
          try { setRecent(await api.recent()); } catch { /* History must not block recovery. */ }
          if (alive) { queuedOpen.current = true; setStartupBusy(false); }
        }
      }
    })();
    return () => { alive = false; for (const dispose of disposers) dispose(); };
  }, []);
  const chooseOpen = () => { if (desktop && !current.current.busy) { setOpen(true); setError(null); } };
  const navigate = (destination: Destination) => {
    const id = tabId(workspace?.id, destination);
    const title = destination === "jobs" ? "Background jobs" : destinations.find(item => item.id === destination)?.label || destination;
    setTabs(previous => openTab(previous, { id, destination, title })); setActive(id);
  };
  const forgetRecent = (path: string | null) => { void run(async () => setRecent(await api.forgetRecent(path))); };
  const closeWorkspace = () => guard(() => { void run(async () => {
    const currentWorkspace = current.current.workspace;
    if (!currentWorkspace) return;
    await api.close({ workspace_id: currentWorkspace.id, revision: currentWorkspace.revision });
    setWorkspace(null); setDirty(false); setTerminalCount(0); setTerminalRequest(0); setRecentOpen(false);
  }); });
  const chooseCreate = () => { if (desktop && !current.current.busy) guard(() => { setCreate(true); setError(null); }); };
  const requestCloseTab = (id: string) => {
    if (busy || (runActive && tabs.length === 1)) { setError("Stop the active run before closing Ghosh."); return; }
    const tab = tabs.find(item => item.id === id);
    const perform = () => {
      if (tabs.length === 1 && tab?.destination === "overview") {
        if (desktop) { allowClose.current = true; void getCurrentWindow().close(); }
        return;
      }
      const result = closeTab(tabs, id, active);
      if (!result.tabs.length) {
        const overview: AppTab = { id: tabId(workspace?.id, "overview"), destination: "overview", title: workspace ? "Overview" : "Welcome" };
        setTabs([overview]); setActive(overview.id); return;
      }
      setTabs(result.tabs); setActive(result.active!);
      requestAnimationFrame(() => document.getElementById(`tab-${result.active}`)?.focus());
    };
    if (tab?.destination === "terminal") confirm("Terminate terminal sessions and close this tab?", perform);
    else if (tab?.destination === "overview" && dirty) confirm("Discard unsaved changes and close this tab?", perform); else perform();
  };
  const execute = (command: Command) => {
    setCommandOpen(false);
    if (command.id === "workspace.open") chooseOpen();
    else if (command.id === "workspace.create") chooseCreate();
    else if (command.id === "workspace.close") closeWorkspace();
    else if (command.id === "workspace.recent") setRecentOpen(true);
    else if (command.id === "workspace.clear_history") forgetRecent(null);
    else if (command.id === "keyboard.open") setKeyboardHelp(true);
    else if (command.id === "navigation.focus") requestAnimationFrame(() => document.querySelector<HTMLButtonElement>("#workspace-navigation button[aria-current=page]")?.focus());
    else if (command.id === "node.add" || command.id === "node.edit" || command.id === "node.test" || command.id === "node.terminal") {
      navigate("nodes"); setNodeRequest({ action: command.id === "node.add" ? "create" : command.id === "node.test" ? "test" : command.id === "node.terminal" ? "terminal" : "edit", sequence: ++nodeSequence.current });
    }
    else if (command.id === "key.system" || command.id === "key.import" || command.id === "key.generate") {
      navigate("keys"); setKeyRequest({ action: command.id === "key.system" ? "system" : command.id === "key.generate" ? "generate" : "import", sequence: ++keySequence.current });
    }
    else if (command.id === "recipe.apply") { navigate("recipes"); setAutomationRequest({ scripts: false, action: "new", sequence: ++automationSequence.current }); }
    else if (command.id.startsWith("playbook.")) {
      const scripts = false; navigate("playbooks");
      setAutomationRequest({ scripts, action: command.id.endsWith(".edit") ? "edit" : command.id.endsWith(".import") ? "import" : "new", sequence: ++automationSequence.current });
    }
    else if (command.id === "terminal.open") { navigate("terminal"); setTerminalRequest(++terminalSequence.current); }
    else if (command.destination) { navigate(command.destination); requestAnimationFrame(() => document.getElementById(`tab-${tabId(workspace?.id, command.destination!)}`)?.focus()); }
  };
  useEffect(() => {
    setTabs([{ id: tabId(workspace?.id, "overview"), destination: "overview", title: workspace ? "Overview" : "Welcome" }]);
    setActive(tabId(workspace?.id, "overview"));
  }, [workspace?.id]);
  useEffect(() => {
    document.documentElement.style.setProperty("--font-size", `${appearance.fontSize}px`);
    document.documentElement.style.setProperty("--ui-scale", String(appearance.zoom / 100));
    document.documentElement.dataset.density = appearance.density;
    // Text zoom changes available columns even when the window's pixel width stays fixed.
    const resize = () => {
      const width = window.innerWidth * 14 / (appearance.fontSize * appearance.zoom / 100);
      for (const [name, limit] of [["small", 1100], ["compact", 800], ["tiny", 540]] as const) {
        document.documentElement.toggleAttribute(`data-layout-${name}`, width <= limit);
      }
    };
    resize(); window.addEventListener("resize", resize);
    return () => window.removeEventListener("resize", resize);
  }, [appearance]);
  const changeAppearance = (value: Appearance) => { setAppearance(value); setPreferenceNotice(saveAppearance(value) ? "" : "Preferences work for this session, but could not be saved."); };
  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (event.defaultPrevented || event.isComposing || event.altKey) return;
      const key = shortcutKey(event);
      const inTerminal = tabs.find(tab => tab.id === active)?.destination === "terminal";
      const cmd = event.metaKey || (event.ctrlKey && !inTerminal);
      const modal = !!document.querySelector("dialog[open]");
      if (cmd && (key === "k" || (key === "p" && event.shiftKey))) { event.preventDefault(); if (!modal) setCommandOpen(true); return; }
      if (modal) return;
      if (cmd && /^[1-9]$/.test(key)) { event.preventDefault(); const tab = tabs[Number(key) - 1]; if (tab) { setActive(tab.id); requestAnimationFrame(() => document.getElementById(`tab-${tab.id}`)?.focus()); } }
      else if (cmd && key === "o") { event.preventDefault(); chooseOpen(); }
      else if (cmd && key === "n") { event.preventDefault(); if (tabs.find(tab => tab.id === active)?.destination === "keys") setKeyRequest({ action: "import", sequence: ++keySequence.current }); else if (!workspace) chooseCreate(); else if (tabs.find(tab => tab.id === active)?.destination === "nodes" && !workspace.read_only && !workspace.external_changed && !busy) { setNodeRequest({ action: "create", sequence: ++nodeSequence.current }); } else setCommandOpen(true); }
      else if (cmd && key === "e" && event.shiftKey) { event.preventDefault(); document.querySelector<HTMLButtonElement>("#workspace-navigation button[aria-current=page]")?.focus(); }
      else if (cmd && (event.code === "BracketLeft" || event.code === "BracketRight" || key === "[" || key === "]")) { event.preventDefault(); if (inTerminal && event.shiftKey) { window.dispatchEvent(new CustomEvent("terminal-shortcut", { detail: event.code === "BracketLeft" || key === "[" ? "previous" : "next" })); return; } const previous = event.code === "BracketLeft" || key === "["; const index = tabs.findIndex(tab => tab.id === active); const next = tabs[(index + (previous ? -1 : 1) + tabs.length) % tabs.length]; setActive(next.id); requestAnimationFrame(() => document.getElementById(`tab-${next.id}`)?.focus()); }
      else if (cmd && key === "w" && event.shiftKey) { event.preventDefault(); if (workspace) closeWorkspace(); }
      else if (cmd && key === "w") { event.preventDefault(); if (inTerminal) window.dispatchEvent(new CustomEvent("terminal-shortcut", { detail: "close" })); else requestCloseTab(active); }
      else if (event.ctrlKey && !event.metaKey && key === "tab") { event.preventDefault(); const index = tabs.findIndex(tab => tab.id === active); const next = tabs[(index + (event.shiftKey ? -1 : 1) + tabs.length) % tabs.length].id; setActive(next); requestAnimationFrame(() => document.getElementById(`tab-${next}`)?.focus()); }
      else if (cmd && key === "s") { event.preventDefault(); if (tabs.find(tab => tab.id === active)?.destination === "overview") document.querySelector<HTMLFormElement>("#workspace-details-form")?.requestSubmit(); }
      else if (cmd && key === "f") { const search = document.querySelector<HTMLInputElement>("section[role=tabpanel]:not([hidden]) [data-surface-search]"); if (search) { event.preventDefault(); search.focus(); search.select(); } }
    };
    window.addEventListener("keydown", handler); return () => window.removeEventListener("keydown", handler);
  });
  return <><ShortcutOverlay /><AppShell workspace={workspace} busy={startupBusy || busy || operations.mutating} activeJobs={operations.snapshot?.jobs.jobs.filter(job => ["running", "cancelling"].includes(job.state)).length || 0} pendingApprovals={operations.snapshot?.approvals.requests.filter(request => request.state === "pending").length || 0} dirty={dirty} tabs={tabs} active={active} navigate={navigate} activate={setActive} closeTab={requestCloseTab} command={() => setCommandOpen(true)} openWorkspace={() => setRecentOpen(true)}>
    {!desktop && <section className="notice" role="status">Desktop preview · local workspace operations require the Ghosh app.</section>}
    {error && <section className="notice error" role="alert"><div><strong>Couldn’t complete that action</strong><p>{error}</p></div><button className="quiet" onClick={() => setError(null)} aria-label="Dismiss error"><Icon name="X" /></button></section>}
    {preferenceNotice && <p role="status" className="notice">{preferenceNotice}</p>}
    {tabs.map(tab => <section key={tab.id} id={`panel-${tab.id}`} role="tabpanel" aria-labelledby={`tab-${tab.id}`} hidden={tab.id !== active}>
      {tab.destination === "overview" ? workspace ? <WorkspaceView key={workspace.id} workspace={workspace} busy={busy} run={action => { void run(action); }} update={accept} dirtyChanged={setDirty} confirm={confirm} /> : <>
        <div className="page-heading"><div><p className="eyebrow">LOCAL CONTROL PLANE</p><h1>Welcome to ghosh<span className="accent">.</span></h1><p>Your nodes, scenarios and browser sessions in one workspace.</p></div></div>
        <section className="panel start-panel"><div className="terminal-prompt"><span>ghosh &gt;</span> workspace open<span className="cursor">▌</span></div><p>Create a portable .ghosh package or open one you already own.</p><div className="inline-actions"><button className="primary" disabled={!desktop || startupBusy || busy} onClick={chooseCreate}><Icon name="Plus" />Create workspace</button><button disabled={!desktop || startupBusy || busy} onClick={chooseOpen}><Icon name="FolderOpen" />Open workspace <kbd>⌘O</kbd></button><button onClick={() => setCommandOpen(true)}>Commands <kbd>⌘K</kbd></button></div><p className="field-help">Opening a package runs nothing. No account required.</p></section>
        <section className="panel"><RecentWorkspaces entries={recent} busy={busy} open={openWorkspace} forget={forgetRecent} /></section>
        <section className="panel"><h2>First workspace workflow</h2><ol className="onboarding"><li><span>01</span> Choose a system or custom SSH key file</li><li><span>02</span> Add a node and verify its host identity</li><li><span>03</span> Open a real terminal or create a YAML scenario</li><li><span>04</span> Review a plan, approve changes and inspect results</li></ol><p className="field-help">These operational services are being connected in the next implementation phases.</p></section>
      </> : tab.destination === "terminal" && desktop ? <TerminalView workspace={workspace} incoming={incomingTerminal} incomingHandled={() => setIncomingTerminal(null)} active={tab.id === active} request={terminalRequest} requestHandled={() => setTerminalRequest(0)} modalChanged={setTerminalModal} sessionCountChanged={setTerminalCount} fontSize={appearance.fontSize * appearance.zoom / 100} /> : tab.destination === "nodes" && workspace ? <NodesView onTerminal={session => { setIncomingTerminal(session); navigate("terminal"); }} workspace={workspace} runActive={runActive} busy={busy || blockingOperation || inventoryBusy} request={nodeRequest} requestHandled={() => setNodeRequest(null)} modalChanged={setInventoryModal} busyChanged={setInventoryBusy} update={setWorkspace} /> : tab.destination === "keys" && desktop ? <KeysView workspace={workspace} update={setWorkspace} request={keyRequest} requestHandled={() => setKeyRequest(null)} modalChanged={setKeysModal} busyChanged={setKeysBusy} /> : ["playbooks", "recipes"].includes(tab.destination) && desktop && workspace ? <AutomationView newSteps={() => { navigate("playbooks"); setAutomationRequest({ scripts: false, action: "new", sequence: ++automationSequence.current }); }} workspace={workspace} scripts={false} recipes={tab.destination === "recipes"} request={tab.id === active ? automationRequest : null} requestHandled={() => setAutomationRequest(null)} modalChanged={setAutomationModal} busyChanged={setAutomationBusy} update={setWorkspace} started={() => { current.current.busy = true; setRunStarting(true); void operations.refresh().finally(() => setRunStarting(false)); navigate("runs"); }} /> : tab.destination === "monitor" && desktop && workspace ? <MonitorView workspace={workspace} active={tab.id === active} busyChanged={setInventoryBusy} modalChanged={setInventoryModal} /> : tab.destination === "runs" && desktop && workspace ? <RunsView workspace={workspace} /> : tab.destination === "jobs" && desktop ? <JobsView operations={operations} /> : tab.destination === "approvals" && desktop ? <ApprovalsView operations={operations} modalChanged={setApprovalModal} /> : tab.destination === "settings" ? <SettingsView appearance={appearance} change={changeAppearance} /> : <div className="empty-state unavailable"><Icon name={destinations.find(item => item.id === tab.destination)?.icon || "Activity"} className="feature-icon" /><p className="eyebrow">{tab.title.toUpperCase()}</p><h1>{tab.title}</h1><p>{!workspace && !["jobs", "approvals"].includes(tab.destination) ? "Open a workspace to use this section." : "The native service for this section is not available in this build."}</p><p className="field-help">No connections, tasks or simulated operations have been started.</p><button onClick={() => setCommandOpen(true)}>Open commands <kbd>⌘K</kbd></button></div>}
    </section>)}
    </AppShell>
    {commandOpen && <CommandPopup dismiss={() => setCommandOpen(false)} execute={execute} context={{ desktop, workspace: !!workspace, runActive, busy: startupBusy || busy || blockingOperation || inventoryBusy || keysBusy || automationBusy, trusted: workspace?.trusted, editable: !!workspace && !workspace.read_only && !workspace.external_changed }} />}
    {recentOpen && <Dialog title="recent-title" dismiss={() => setRecentOpen(false)} busy={busy}>
      <h2 id="recent-title">Workspaces</h2>
      <div className="inline-actions"><button data-dialog-focus autoFocus disabled={busy} onClick={() => { setRecentOpen(false); chooseCreate(); }}><Icon name="Plus" />New workspace</button><button disabled={busy} onClick={() => { setRecentOpen(false); chooseOpen(); }}><Icon name="FolderOpen" />Open workspace</button>{workspace && <button disabled={busy} onClick={closeWorkspace}>Close workspace</button>}</div>
      <RecentWorkspaces entries={recent} busy={busy} open={openWorkspace} forget={forgetRecent} />
      <div className="modal-actions"><button disabled={busy} onClick={() => setRecentOpen(false)}>Done</button></div>
    </Dialog>}
    {keyboardHelp && <Dialog title="keyboard-title" dismiss={() => setKeyboardHelp(false)} className="keyboard-dialog"><h2 id="keyboard-title">Keyboard shortcuts</h2><SettingsView appearance={appearance} change={changeAppearance} keyboard /><div className="modal-actions"><button autoFocus data-dialog-focus onClick={() => setKeyboardHelp(false)}>Close</button></div></Dialog>}
    {create && <Dialog title="create-title" dismiss={() => setCreate(false)} busy={busy}><button className="modal-close quiet" aria-label="Close create dialog" disabled={busy} onClick={() => setCreate(false)}><Icon name="X" /></button><p className="eyebrow">MAKE IT YOURS</p><h2 id="create-title">Create a workspace</h2><p>A local package for your infrastructure. You can move it later.</p><form onSubmit={e => { e.preventDefault(); void run(async () => { accept(await api.create({ parent: location, name: name.trim() })); setCreate(false); setName(""); }); }}><label htmlFor="new-name">Workspace name</label><input autoFocus data-dialog-focus id="new-name" placeholder="e.g. Homelab" maxLength={120} value={name} disabled={busy} onChange={e => setName(e.target.value)} required /><label htmlFor="new-location">Location</label><div className="input-row"><input id="new-location" placeholder="Choose a parent folder" value={location} disabled={busy} onChange={e => setLocation(e.target.value)} required /><button type="button" disabled={busy} onClick={() => { void run(async () => { const path = await api.pick("parent"); if (path) setLocation(path); }); }}>Browse…</button></div>{error && <p className="form-error" role="alert">{error}</p>}<p className="field-help">Creates {name.trim() || "Your workspace"}.ghosh in this folder.</p><div className="modal-actions"><button type="button" disabled={busy} onClick={() => setCreate(false)}>Cancel</button><button className="primary" disabled={busy || !name.trim() || !location.trim()}>Create workspace</button></div></form></Dialog>}
    {open && <Dialog title="open-title" dismiss={() => setOpen(false)} busy={busy}><button className="modal-close quiet" aria-label="Close open dialog" disabled={busy} onClick={() => setOpen(false)}><Icon name="X" /></button><p className="eyebrow">WELCOME BACK</p><h2 id="open-title">Open a workspace</h2><p>Choose an existing .ghosh package on your machine.</p><form onSubmit={e => { e.preventDefault(); openWorkspace(openPath); }}><label htmlFor="open-path">Workspace location</label><div className="input-row"><input autoFocus data-dialog-focus id="open-path" placeholder="/path/to/Homelab.ghosh" value={openPath} onChange={e => setOpenPath(e.target.value)} disabled={busy} required/><button type="button" disabled={busy} onClick={() => { void run(async () => { const path = await api.pick("workspace"); if (path) setOpenPath(path); }); }}>Browse…</button></div>{error && <p className="form-error" role="alert">{error}</p>}<p className="field-help">Opening a package does not run its scripts or hooks.</p><div className="modal-actions"><button type="button" disabled={busy} onClick={() => setOpen(false)}>Cancel</button><button className="primary" disabled={busy || !openPath.trim()}>Open workspace</button></div></form></Dialog>}
    {confirmation && <Dialog title="confirm-title" dismiss={() => setConfirmation(null)} busy={busy} alert><h2 id="confirm-title">{confirmation.title}</h2><p>{confirmation.title.startsWith("Terminate") ? "Terminal sessions and their foreground jobs will be terminated. Unsaved terminal work and workspace drafts may be lost." : confirmation.title.startsWith("Trust") ? "Trust applies to this session and this exact configuration. Node connection tests require a separate identity and server-key review." : "Saved files will stay as they are. Your unsaved changes will be discarded when you continue."}</p><div className="modal-actions"><button autoFocus data-dialog-focus onClick={() => setConfirmation(null)}>Cancel</button><button className="primary" onClick={() => { const action = confirmation.action; setConfirmation(null); action(); }}>Continue</button></div></Dialog>}
  </>;
}
