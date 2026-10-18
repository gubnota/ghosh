import { useEffect, useRef, type ReactNode } from "react";
const invokers = new WeakMap<HTMLDialogElement, HTMLElement | null>();
function activeInvoker(): HTMLElement | null {
  const active = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  const dialog = active?.closest("dialog");
  return dialog && invokers.has(dialog) ? invokers.get(dialog)! : active;
}

export function Dialog({ title, dismiss, busy = false, alert = false, className = "", children }: {
  title: string; dismiss: () => void; busy?: boolean; alert?: boolean; className?: string; children: ReactNode;
}) {
  const element = useRef<HTMLDialogElement>(null);
  // Capture before child autofocus runs during the commit.
  const invoker = useRef(activeInvoker());
  useEffect(() => {
    const dialog = element.current!;
    invokers.set(dialog, invoker.current);
    dialog.showModal();
    // React autofocus runs before showModal; WebKit then chooses the first control.
    dialog.querySelector<HTMLElement>("[data-dialog-focus]")?.focus();
    return () => { dialog.close(); if (invoker.current?.isConnected) invoker.current.focus(); };
  }, []);
  useEffect(() => {
    const dialog = element.current!;
    // A review may mount while its validation task is finishing. Its initial
    // control is disabled then; give it focus when the task releases the UI.
    if (!busy && dialog.open && (document.activeElement === dialog || !dialog.contains(document.activeElement))) {
      dialog.querySelector<HTMLElement>("[data-dialog-focus]:not(:disabled)")?.focus();
    }
  }, [busy]);
  return <dialog ref={element} className={`modal ${className}`} role={alert ? "alertdialog" : "dialog"} aria-labelledby={title} onKeyDown={event => {
    if (event.key !== "Tab") return;
    const controls = [...event.currentTarget.querySelectorAll<HTMLElement>('button, input, select, textarea, a[href], [tabindex]')].filter(control => control.tabIndex >= 0 && !control.matches(":disabled") && control.getClientRects().length);
    const first = controls[0], last = controls[controls.length - 1];
    if (!first) { event.preventDefault(); return; }
    if (event.shiftKey && (document.activeElement === first || document.activeElement === event.currentTarget)) { event.preventDefault(); last.focus(); }
    else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
  }}
    onCancel={event => { event.preventDefault(); if (!busy) dismiss(); }}>{children}</dialog>;
}
