import { Icon } from "../../components/Icon";
import { useEffect, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { SearchAddon } from "@xterm/addon-search";
import "@xterm/xterm/css/xterm.css";
import { api, errorMessage } from "../../app/ipc";
import { Dialog } from "../../components/Dialog";
import type { TerminalSession } from "../../generated/contracts";

const theme = { background: "#011627", foreground: "#bdc1c6", cursor: "#9ca1aa", selectionBackground: "#b2ceee66", black: "#1d3b53", red: "#fc514e", green: "#a1cd5e", yellow: "#e7d37a", blue: "#82aaff", magenta: "#c792ea", cyan: "#7fdbca", white: "#a1aab8", brightBlack: "#7c8f8f", brightRed: "#ff5874", brightGreen: "#21c7a8", brightYellow: "#ecc48d", brightBlue: "#82aaff", brightMagenta: "#ae81ff", brightCyan: "#7fdbca", brightWhite: "#d6deeb" };
type PaneHandle = { focus: () => void; paste: (value: string) => void; find: (value: string, previous?: boolean) => void };

function TerminalPane({ session, visible, fontSize, register, reviewPaste, report }: {
  session: TerminalSession; visible: boolean; fontSize: number; register: (id: string, handle: PaneHandle | null) => void;
  reviewPaste: (id: string, text: string) => void; report: (message: string) => void;
}) {
  const host = useRef<HTMLDivElement>(null);
  const terminal = useRef<Terminal | null>(null);
  const fit = useRef<FitAddon | null>(null);
  const [status, setStatus] = useState("Connected");
  const callbacks = useRef({ reviewPaste, report }); callbacks.current = { reviewPaste, report };
  useEffect(() => {
    let alive = true, timer = 0, ack = 0, stopped = false, writing = false;
    const queue: Uint8Array[] = [];
    const term = new Terminal({ theme, fontFamily: '"SFMono-Regular", Menlo, Monaco, monospace', fontSize, lineHeight: 1.2, scrollback: 5000, allowProposedApi: false, screenReaderMode: true, convertEol: false });
    const fitter = new FitAddon(), search = new SearchAddon();
    terminal.current = term; fit.current = fitter;
    term.loadAddon(fitter); term.loadAddon(search); term.open(host.current!);
    // Reserve only app shortcuts; Ctrl+C/D/Z and ordinary shell keys reach the PTY.
    term.attachCustomKeyEventHandler(event => !(event.metaKey || (event.ctrlKey && event.key === "Tab")));
    const drain = async () => {
      if (writing || stopped) return;
      writing = true;
      try {
        while (alive && queue.length) {
          try { await api.terminalInput(session.id, Array.from(queue[0])); queue.shift(); }
          catch (error) {
            if ((error as { code?: string })?.code === "capacity") { await new Promise(resolve => setTimeout(resolve, 30)); continue; }
            throw error;
          }
        }
      } catch (error) { stopped = true; setStatus("Input unavailable"); callbacks.current.report(errorMessage(error)); }
      finally { writing = false; }
    };
    const input = term.onData(value => {
      if (stopped) return;
      const bytes = new TextEncoder().encode(value);
      if (bytes.length > 65536 || queue.reduce((sum, part) => sum + part.length, 0) + bytes.length > 65536) {
        callbacks.current.report("Terminal input is busy. Wait before typing or pasting more."); return;
      }
      for (let i = 0; i < bytes.length; i += 16384) queue.push(bytes.slice(i, i + 16384));
      void drain();
    });
    const interceptPaste = (event: ClipboardEvent) => {
      event.preventDefault(); event.stopImmediatePropagation();
      const text = event.clipboardData?.getData("text/plain") || "";
      if (new TextEncoder().encode(text).byteLength > 32768) { callbacks.current.report("Paste is limited to 32 KiB of text."); return; }
      if (/[\r\n]/.test(text) || text.length > 256) callbacks.current.reviewPaste(session.id, text);
      else term.paste(text);
    };
    host.current!.addEventListener("paste", interceptPaste, true);
    register(session.id, { focus: () => term.focus(), paste: value => term.paste(value), find: (value, previous) => { if (previous) search.findPrevious(value); else search.findNext(value); } });
    const poll = async () => {
      if (!alive || stopped) return;
      try {
        const packet = await api.terminalPoll(session.id, ack);
        if (!alive) return;
        if (packet.sequence !== ack) {
          // Acknowledge only after the emulator parsed this packet. No output is
          // dropped when rendering falls behind; native queue pressure reaches PTY.
          if (packet.bytes.length) await new Promise<void>(resolve => term.write(new Uint8Array(packet.bytes), resolve));
          ack = packet.sequence;
          if (packet.exited || packet.failed) { stopped = true; setStatus(packet.failed ? "Terminal transport failed" : `Exited · ${packet.exit_code ?? "?"}`); return; }
        }
        if (alive) timer = window.setTimeout(() => { void poll(); }, packet.bytes.length ? 0 : 40);
      } catch (error) { if (alive) { stopped = true; setStatus("Disconnected"); callbacks.current.report(errorMessage(error)); } }
    };
    const resize = term.onResize(({ rows, cols }) => { if (!stopped) void api.terminalResize(session.id, { rows, cols }).catch(error => callbacks.current.report(errorMessage(error))); });
    const observer = new ResizeObserver(() => { if (host.current?.getClientRects().length) fitter.fit(); }); observer.observe(host.current!);
    void poll();
    const element = host.current!;
    return () => { alive = false; stopped = true; window.clearTimeout(timer); observer.disconnect(); element.removeEventListener("paste", interceptPaste, true); input.dispose(); resize.dispose(); register(session.id, null); term.dispose(); terminal.current = null; };
  }, [session.id]);
  useEffect(() => { if (terminal.current) { terminal.current.options.fontSize = fontSize; if (visible) fit.current?.fit(); } }, [fontSize, visible]);
  return <section className="terminal-pane" hidden={!visible} aria-label={`${session.target} terminal`}><div className="terminal-status"><span>{session.target} · {session.shell}</span><span role="status">{status}</span></div><div className="terminal-emulator" ref={host} /></section>;
}

export function TerminalView({ active, request, requestHandled, modalChanged, sessionCountChanged, fontSize }: {
  active: boolean; request: number; requestHandled: () => void; modalChanged: (open: boolean) => void; sessionCountChanged: (count: number) => void; fontSize: number;
}) {
  const [sessions, setSessions] = useState<TerminalSession[]>([]);
  const sessionsRef = useRef<TerminalSession[]>([]); sessionsRef.current = sessions;
  const [selected, setSelected] = useState("");
  const [split, setSplit] = useState(false);
  const [opening, setOpening] = useState(false), [busy, setBusy] = useState(false);
  const [closing, setClosing] = useState<string | null>(null);
  const [paste, setPaste] = useState<{ id: string; text: string } | null>(null);
  const [error, setError] = useState("");
  const [query, setQuery] = useState("");
  const handles = useRef(new Map<string, PaneHandle>());
  const alive = useRef(true);
  useEffect(() => { alive.current = true; return () => { alive.current = false; for (const session of sessionsRef.current) void api.terminalClose(session.id).catch(() => {}); }; }, []);
  useEffect(() => { modalChanged(opening || !!closing || !!paste); return () => modalChanged(false); }, [opening, closing, paste, modalChanged]);
  useEffect(() => { if (request) { setOpening(true); requestHandled(); } }, [request]);
  useEffect(() => { sessionCountChanged(sessions.length); return () => sessionCountChanged(0); }, [sessions.length, sessionCountChanged]);
  const focus = (id: string) => {
    const handle = handles.current.get(id);
    if (handle) handle.focus();
    else requestAnimationFrame(() => handles.current.get(id)?.focus());
  };
  const open = async () => {
    if (busy) return; setBusy(true); setError("");
    try {
      const session = await api.terminalOpen({ rows: 24, cols: 80 });
      if (!alive.current) { await api.terminalClose(session.id); return; }
      setSessions(previous => [...previous, session]); setSelected(session.id); setOpening(false); focus(session.id);
    } catch (error) { setError(errorMessage(error)); } finally { if (alive.current) setBusy(false); }
  };
  const close = async (id: string) => {
    if (busy) return; setBusy(true);
    try { await api.terminalClose(id); setSessions(previous => previous.filter(session => session.id !== id)); setSelected(sessions.find(session => session.id !== id)?.id || ""); setClosing(null); }
    catch (error) { setError(errorMessage(error)); } finally { setBusy(false); }
  };
  const secondary = sessions.find(session => session.id !== selected)?.id;
  return <div className="terminal-surface"><div className="page-heading"><div><p className="eyebrow">REAL PTY</p><h1>Terminal</h1><p>Local shell · Cmd+K commands · Cmd+Shift+E leaves terminal focus</p></div><button onClick={() => setOpening(true)} disabled={sessions.length >= 8}><Icon name="Plus" />New local terminal</button></div>
    {error && <p className="form-error" role="alert">{error}</p>}
    <div className="terminal-toolbar"><label htmlFor="terminal-search" className="sr-only">Search terminal output</label><input id="terminal-search" data-surface-search placeholder="Search output…" value={query} onChange={event => setQuery(event.target.value)} onKeyDown={event => { if (event.key === "Enter") { event.preventDefault(); handles.current.get(selected)?.find(query, event.shiftKey); } }} /><button onClick={() => handles.current.get(selected)?.find(query)}><Icon name="Search" />Find next</button><button disabled={sessions.length < 2} aria-pressed={split} onClick={() => setSplit(value => !value)}>Split view</button><button disabled={!selected} onClick={() => focus(selected)}>Focus terminal</button><button disabled={!selected} onClick={() => setClosing(selected)}><Icon name="X" />Close session</button></div>
    <div className="terminal-sessions" role="group" aria-label="Terminal sessions">{sessions.map((session, index) => <button key={session.id} aria-pressed={selected === session.id} onClick={() => { setSelected(session.id); focus(session.id); }}>Local {index + 1}</button>)}</div>
    {!sessions.length && <div className="empty-state"><h2>No terminal sessions</h2><p>Open a real local shell. Node SSH sessions follow in the next phase.</p><button onClick={() => setOpening(true)}><Icon name="Terminal" />Open local terminal</button></div>}
    <div className={`terminal-panes ${split ? "split" : ""}`}>{sessions.map(session => <TerminalPane key={session.id} session={session} visible={active && (selected === session.id || (split && secondary === session.id))} fontSize={fontSize} register={(id, handle) => { if (handle) handles.current.set(id, handle); else handles.current.delete(id); }} reviewPaste={(id, text) => setPaste({ id, text })} report={setError} />)}</div>
    {opening && <Dialog title="terminal-open-title" dismiss={() => setOpening(false)} busy={busy}><p className="eyebrow">LOCAL MACHINE</p><h2 id="terminal-open-title">Open local terminal?</h2><p>This starts your login shell in your home directory, with your normal startup files and permissions. Commands execute immediately. Your shell may save its own history.</p><p>Ghosh keeps up to 5,000 scrollback lines in memory and does not save transcripts. Closing this tab terminates its sessions.</p>{error && <p role="alert">{error}</p>}<div className="modal-actions"><button data-dialog-focus disabled={busy} onClick={() => setOpening(false)}>Cancel</button><button className="primary" disabled={busy} onClick={() => { void open(); }}>Open shell</button></div></Dialog>}
    {closing && <Dialog title="terminal-close-title" dismiss={() => setClosing(null)} busy={busy} alert><h2 id="terminal-close-title">Terminate terminal session?</h2><p>The shell and its foreground job will receive a hangup. Unsaved work in terminal programs may be lost.</p><div className="modal-actions"><button data-dialog-focus disabled={busy} onClick={() => setClosing(null)}>Cancel</button><button disabled={busy} onClick={() => { void close(closing); }}>Terminate session</button></div></Dialog>}
    {paste && <Dialog title="terminal-paste-title" dismiss={() => { setPaste(null); focus(paste.id); }}><h2 id="terminal-paste-title">Review terminal paste</h2><p>Multiline text may execute commands immediately.</p><pre className="paste-preview">{paste.text}</pre><div className="modal-actions"><button data-dialog-focus onClick={() => { setPaste(null); focus(paste.id); }}>Cancel</button><button className="primary" onClick={() => { handles.current.get(paste.id)?.paste(paste.text); setPaste(null); focus(paste.id); }}>Paste into terminal</button></div></Dialog>}
  </div>;
}
