import type { Destination } from "./commands";
export type AppTab = { id: string; destination: Destination; title: string };
export function tabId(workspaceId: string | undefined, destination: Destination): string {
  return `${workspaceId || "app"}:${destination}`;
}
export function openTab(tabs: AppTab[], tab: AppTab): AppTab[] {
  return tabs.some(existing => existing.id === tab.id) ? tabs : [...tabs, tab];
}
export function closeTab(tabs: AppTab[], id: string, active: string): { tabs: AppTab[]; active: string | null } {
  const index = tabs.findIndex(tab => tab.id === id);
  const remaining = tabs.filter(tab => tab.id !== id);
  return { tabs: remaining, active: active !== id ? active : remaining[Math.min(index, remaining.length - 1)]?.id || null };
}
