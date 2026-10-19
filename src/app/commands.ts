import type { IconName } from "../components/Icon";
export type Destination = "terminal" | "overview" | "nodes" | "playbooks" | "runs" | "keys" | "browser" | "tunnels" | "certificates" | "scripts" | "recordings" | "recipes" | "mcp" | "approvals" | "settings" | "jobs";
export const destinations: { id: Destination; label: string; group: string; icon: IconName; description: string }[] = [
  { id: "overview", label: "Overview", group: "Workspace", icon: "LayoutDashboard", description: "Workspace configuration, trust and inventory" },
  { id: "nodes", label: "Nodes", group: "Infrastructure", icon: "Server", description: "Search and select configured nodes" },
  { id: "terminal", label: "Terminal", group: "Infrastructure", icon: "Terminal", description: "Real local and node PTY sessions" },
  { id: "playbooks", label: "Playbooks", group: "Infrastructure", icon: "BookText", description: "YAML scenarios, targets and task definitions" },
  { id: "runs", label: "Runs", group: "Infrastructure", icon: "History", description: "Plans, execution results and history" },
  { id: "keys", label: "SSH Keys", group: "Infrastructure", icon: "KeyRound", description: "System and custom file-based identities" },
  { id: "browser", label: "Browser Profiles", group: "Automation", icon: "Globe", description: "Isolated browser profiles and sessions" },
  { id: "tunnels", label: "Tunnels", group: "Automation", icon: "Route", description: "Routes, consumers and connection health" },
  { id: "certificates", label: "Certificates", group: "Automation", icon: "ShieldCheck", description: "Issuance, renewal and distribution" },
  { id: "scripts", label: "Scripts", group: "Automation", icon: "FileCode", description: "Author and run reviewed scripts" },
  { id: "recordings", label: "Recordings", group: "Automation", icon: "MonitorPlay", description: "Browser capture, editing and replay" },
  { id: "recipes", label: "ComfyUI Recipes", group: "Automation", icon: "Cpu", description: "Pinned installations and verified models" },
  { id: "mcp", label: "MCP", group: "System", icon: "Plug", description: "Agent tools and external clients" },
  { id: "approvals", label: "Approvals", group: "System", icon: "Inbox", description: "Review and authorize pending operations" },
  { id: "settings", label: "Settings", group: "System", icon: "Settings", description: "Appearance, keyboard and application preferences" },
];
export type CommandId = "workspace.open" | "workspace.create" | "workspace.close" | "workspace.recent" | "workspace.clear_history" | "node.add" | "node.edit" | "node.test" | "node.terminal" | "key.system" | "key.import" | "key.generate" | "playbook.new" | "playbook.import" | "playbook.edit" | "script.new" | "script.import" | "script.edit" | "run.plan" | "run.apply" | "terminal.open" | "browser.open" | "route.switch" | "certificate.renew" | "recipe.apply" | "mcp.settings" | "jobs.open" | "approvals.open" | "settings.open" | "keyboard.open" | "navigation.focus";
export type Command = { id: CommandId; label: string; syntax: string; group: string; destination?: Destination; shortcut?: string };
export const commands: Command[] = [
  { id: "workspace.open", label: "Open workspace", syntax: "workspace open", group: "Workspace", shortcut: "⌘O" },
  { id: "workspace.create", label: "Create workspace", syntax: "workspace create", group: "Workspace" },
  { id: "workspace.close", label: "Close workspace", syntax: "workspace close", group: "Workspace", shortcut: "⌘⇧W" },
  { id: "workspace.recent", label: "Recent workspaces", syntax: "workspace recent", group: "Workspace" },
  { id: "workspace.clear_history", label: "Clear workspace history", syntax: "workspace clear history", group: "Workspace" },
  { id: "node.add", label: "Add node", syntax: "node add", group: "Nodes", destination: "nodes", shortcut: "⌘N" },
  { id: "node.edit", label: "Edit node", syntax: "node edit", group: "Nodes", destination: "nodes" },
  { id: "node.terminal", label: "Open node terminal", syntax: "node terminal", group: "Nodes", destination: "nodes" },
  { id: "node.test", label: "Test connection", syntax: "node test", group: "Nodes", destination: "nodes" },
  { id: "key.system", label: "Use system SSH key", syntax: "key system", group: "SSH Keys", destination: "keys" },
  { id: "key.import", label: "Import custom key", syntax: "key import", group: "SSH Keys", destination: "keys" },
  { id: "key.generate", label: "Generate SSH key", syntax: "key generate", group: "SSH Keys", destination: "keys" },
  { id: "playbook.new", label: "New playbook (scenario)", syntax: "playbook new", group: "Playbooks", destination: "playbooks" },
  { id: "playbook.import", label: "Import playbook", syntax: "playbook import", group: "Playbooks", destination: "playbooks" },
  { id: "playbook.edit", label: "Edit playbook", syntax: "playbook edit", group: "Playbooks", destination: "playbooks" },
  { id: "script.new", label: "New script", syntax: "script new", group: "Scripts", destination: "scripts" },
  { id: "script.import", label: "Import script", syntax: "script import", group: "Scripts", destination: "scripts" },
  { id: "script.edit", label: "Edit script", syntax: "script edit", group: "Scripts", destination: "scripts" },
  { id: "run.plan", label: "Plan run", syntax: "run plan", group: "Runs", destination: "runs" },
  { id: "run.apply", label: "Apply run", syntax: "run apply", group: "Runs", destination: "runs" },
  { id: "terminal.open", label: "Open terminal", syntax: "terminal open", group: "Terminal", destination: "terminal" },
  { id: "browser.open", label: "Open browser tab", syntax: "browser open", group: "Browser", destination: "browser" },
  { id: "route.switch", label: "Switch route", syntax: "route switch", group: "Routes", destination: "tunnels" },
  { id: "certificate.renew", label: "Renew certificate", syntax: "certificate renew", group: "Certificates", destination: "certificates" },
  { id: "recipe.apply", label: "Apply ComfyUI recipe", syntax: "recipe apply", group: "Recipes", destination: "recipes" },
  { id: "mcp.settings", label: "MCP server settings", syntax: "mcp settings", group: "System", destination: "mcp" },
  { id: "jobs.open", label: "Background jobs", syntax: "jobs", group: "System", destination: "jobs" },
  { id: "approvals.open", label: "Approvals inbox", syntax: "approvals", group: "System", destination: "approvals" },
  { id: "settings.open", label: "Settings", syntax: "settings", group: "System", destination: "settings" },
  { id: "keyboard.open", label: "Keyboard shortcuts", syntax: "keyboard", group: "System" },
  { id: "navigation.focus", label: "Move focus to navigation", syntax: "navigation", group: "System", shortcut: "⌘⇧E" },
];

// Search is descriptive. Arguments are only accepted by an explicitly implemented
// parser; never drop a suffix and accidentally dispatch a different operation.
export function searchCommands(query: string): Command[] {
  const terms = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  const exact = query.trim().toLowerCase();
  return commands.filter(command => terms.every(term => `${command.label} ${command.syntax} ${command.group}`.toLowerCase().includes(term)))
    .sort((a, b) => Number(b.syntax === exact || b.label.toLowerCase() === exact) - Number(a.syntax === exact || a.label.toLowerCase() === exact));
}
export function commandReason(command: Command, context: { desktop: boolean; workspace: boolean; busy: boolean; editable?: boolean; trusted?: boolean }): string | null {
  if (context.busy) return "Wait for the current operation to finish.";
  if (["settings.open", "keyboard.open", "navigation.focus"].includes(command.id)) return null;
  if (["terminal.open", "workspace.open", "workspace.create", "workspace.recent", "workspace.clear_history", "jobs.open", "approvals.open", "key.system", "key.import", "key.generate"].includes(command.id)) return context.desktop ? null : "Requires the Ghosh desktop app.";
  if (!context.workspace) return "Open a workspace first.";
  if (command.id === "workspace.close") return context.desktop ? null : "Requires the Ghosh desktop app.";
  if (command.id === "node.test" || command.id === "node.terminal") return !context.desktop ? "Requires the Ghosh desktop app." : !context.editable ? "Reload this workspace before testing." : !context.trusted ? "Workspace operations are paused. Resume them in Overview." : null;
  if (["node.add", "node.edit"].includes(command.id)) return !context.desktop ? "Requires the Ghosh desktop app." : context.editable ? null : "Reload this workspace before editing inventory.";
  if (command.id === "run.plan" || command.id === "run.apply") return !context.desktop ? "Requires the Ghosh desktop app." : !context.editable ? "Reload the workspace first." : !context.trusted ? "Resume workspace operations in Overview." : null;
  if (command.id.startsWith("playbook.") || command.id.startsWith("script.")) return !context.desktop ? "Requires the Ghosh desktop app." : context.editable ? null : "Reload this workspace before editing automation.";
  return `${command.group} service is not available in this build.`;
}
