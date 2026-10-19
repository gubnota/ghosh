import { BackgroundRun } from "../features/automation/useRuns";
import { APP_VERSION } from "./version";
import { Icon } from "../components/Icon";
import type { ReactNode } from "react";
import type { WorkspaceSnapshot } from "../generated/contracts";
import { destinations, type Destination } from "./commands";
import type { AppTab } from "./tabs";

export function AppShell({ workspace, busy, activeJobs, pendingApprovals, dirty, active, tabs, navigate, activate, closeTab, command, openWorkspace, children }: {
  workspace: WorkspaceSnapshot | null; busy: boolean; activeJobs: number; pendingApprovals: number; dirty: boolean; active: string; tabs: AppTab[];
  navigate: (destination: Destination) => void; activate: (id: string) => void; closeTab: (id: string) => void;
  command: () => void; openWorkspace: () => void; children: ReactNode;
}) {
  const destination = tabs.find(tab => tab.id === active)?.destination || "overview";
  return <div className="app-shell" aria-busy={busy}>
    <header className="topbar"><button className="brand quiet" onClick={() => navigate("overview")} aria-label="Ghosh overview">g<span>ghosh<span className="accent">.</span></span></button>
      <span className="breadcrumb">/ {workspace?.name || "No workspace"}</span>
      <button className="command-trigger" onClick={command}><span><b>ghosh &gt;</b> <span className="command-placeholder">Type a command…</span></span><kbd>⌘K</kbd></button>
      <button className="quiet header-icon" onClick={() => navigate("jobs")} aria-label="Background jobs" title="Background jobs"><Icon name="Activity" />{activeJobs > 0 && <span className="count-badge">{activeJobs}</span>}</button>
      <button className="quiet header-icon" onClick={() => navigate("approvals")} aria-label="Approvals inbox" title="Approvals inbox"><Icon name="Inbox" />{pendingApprovals > 0 && <span className="count-badge">{pendingApprovals}</span>}</button>
    </header>
    <aside className="sidebar"><button className="workspace-switch" onClick={openWorkspace}><span className="workspace-avatar">{workspace?.name.slice(0, 1).toUpperCase() || "+"}</span><span><strong>{workspace?.name || "Open workspace"}</strong><small>{workspace ? "Local workspace" : "Create or open a .ghosh package"}</small></span><Icon name="ChevronDown" /></button>
      <nav aria-label="Workspace navigation" id="workspace-navigation">{["Workspace", "Infrastructure", "Automation", "System"].map(group => <div className="nav-group" key={group}><p className="nav-caption">{group}</p>{destinations.filter(item => item.group === group).map(item => <button key={item.id} className={`nav-item ${destination === item.id ? "selected" : ""}`} aria-current={destination === item.id ? "page" : undefined} onClick={() => navigate(item.id)}><Icon name={item.icon} /><span className="nav-label">{item.label}</span></button>)}</div>)}</nav>
      <div className="sidebar-bottom"><span className="local-indicator"><i />Local by design</span><button className="quiet" onClick={command}>Commands <kbd>⌘K</kbd></button></div>
    </aside>
    <div className="main-shell"><div className="tab-strip" role="tablist" aria-label="Workspace tabs" onKeyDown={event => {
      if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
      if (!(event.target instanceof HTMLElement) || event.target.getAttribute("role") !== "tab") return;
      event.preventDefault(); const index = tabs.findIndex(tab => tab.id === active);
      const next = event.key === "Home" ? 0 : event.key === "End" ? tabs.length - 1 : (index + (event.key === "ArrowRight" ? 1 : -1) + tabs.length) % tabs.length;
      if (tabs[next]) { activate(tabs[next].id); document.getElementById(`tab-${tabs[next].id}`)?.focus(); }
    }}>{tabs.map(tab => <div className={`tab-item ${tab.id === active ? "active" : ""}`} key={tab.id}><button id={`tab-${tab.id}`} role="tab" aria-selected={tab.id === active} aria-controls={`panel-${tab.id}`} tabIndex={tab.id === active ? 0 : -1} onClick={() => activate(tab.id)}>{tab.title}{tab.destination === "overview" && dirty ? " •" : ""}</button><button className="tab-close" tabIndex={tab.id === active ? 0 : -1} aria-label={`Close ${tab.title} tab`} onClick={() => closeTab(tab.id)}><Icon name="X" /></button></div>)}</div>
      <main className="content" id="main-content" tabIndex={-1}>{children}</main>
    </div>
    <footer className="statusbar"><BackgroundRun workspace={workspace} open={() => navigate("runs")} /><span><i />{busy ? "Working…" : "Ready"}</span><span>{workspace ? `${workspace.trusted ? "Approved" : "Paused"} · ${workspace.external_changed ? "Changed on disk" : workspace.read_only ? "Read-only" : "Local files"}` : "No workspace open"}</span><span className="status-end">{dirty ? "Unsaved changes · " : ""}Ghosh {APP_VERSION} · <button className="quiet" onClick={command}>⌘K commands</button></span></footer>
  </div>;
}
