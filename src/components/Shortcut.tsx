export function Shortcut({ keys }: { keys: string }) {
  return <kbd className="shortcut" aria-label={keys.replaceAll("⇧", "Shift+").replaceAll("⌘", "Command+")}>
    {keys.split(/(⇧|Shift\+)/).map((part, index) => part === "⇧" || part === "Shift+" ? <svg key={index} className="shift-key" viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinejoin="round" aria-hidden="true"><path d="m10 3 7 7h-4v7H7v-7H3z" /></svg> : <span key={index} aria-hidden="true">{part}</span>)}
  </kbd>;
}
