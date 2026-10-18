import { Activity, Inbox, LayoutDashboard, Server, BookText, History, KeyRound, Globe, Route, ShieldCheck, FileCode, MonitorPlay, Cpu, Plug, Settings, Terminal, ChevronDown, X, Plus, FolderOpen, ArrowUpRight, Upload, Play, Pencil, Trash2, Search, Keyboard, Info } from "lucide-react";
const icons = { Activity, Inbox, LayoutDashboard, Server, BookText, History, KeyRound, Globe, Route, ShieldCheck, FileCode, MonitorPlay, Cpu, Plug, Settings, Terminal, ChevronDown, X, Plus, FolderOpen, ArrowUpRight, Upload, Play, Pencil, Trash2, Search, Keyboard, Info };
export type IconName = keyof typeof icons;
export function Icon({ name, className = "" }: { name: IconName; className?: string }) {
  const Component = icons[name];
  return <Component aria-hidden="true" className={`icon ${className}`} />;
}
