// Application shortcuts follow physical keys, including under Russian layouts.
// Text entry and terminal control sequences continue to use the browser's key.
export function shortcutKey(event: Pick<KeyboardEvent, "key" | "code">): string {
  if (/^Key[A-Z]$/.test(event.code)) return event.code.slice(3).toLowerCase();
  if (/^Digit[1-9]$/.test(event.code)) return event.code.slice(5);
  if (event.code === "BracketLeft") return "[";
  if (event.code === "BracketRight") return "]";
  return event.key.toLowerCase();
}
