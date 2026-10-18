import { Children, forwardRef, isValidElement, useEffect, useId, useLayoutEffect, useRef, useState, type ButtonHTMLAttributes, type ReactNode } from "react";
import { Check, ChevronDown } from "lucide-react";

type OptionProps = { value?: string | number; disabled?: boolean; children?: ReactNode };
type Props = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "value" | "onChange" | "children"> & {
  value: string | number; children: ReactNode; onChange: (event: { target: { value: string } }) => void;
};
function textOf(node: ReactNode): string {
  return Children.toArray(node).map(child => isValidElement<OptionProps>(child) ? textOf(child.props.children) : String(child)).join("");
}

/** A keyboard combobox whose menu shares the app theme, including inside dialogs. */
export const Select = forwardRef<HTMLButtonElement, Props>(function Select({ value, children, onChange, className = "", disabled, ...props }, forwardedRef) {
  const options = Children.toArray(children).filter(isValidElement<OptionProps>).map(child => ({
    value: String(child.props.value ?? textOf(child.props.children)), label: textOf(child.props.children), disabled: !!child.props.disabled,
  }));
  const selected = options.findIndex(option => option.value === String(value));
  const uid = useId(), listId = `${uid}-list`;
  const trigger = useRef<HTMLButtonElement>(null), menu = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false), [active, setActive] = useState(selected);
  const typed = useRef({ text: "", at: 0 });
  const enabled = options.map((option, index) => option.disabled ? -1 : index).filter(index => index >= 0);
  const close = () => setOpen(false);
  const reveal = (index = selected) => { if (!trigger.current?.matches(":disabled") && enabled.length) { setActive(enabled.includes(index) ? index : enabled[0]); setOpen(true); } };
  const choose = (index: number) => {
    const option = options[index];
    if (!option || option.disabled || trigger.current?.matches(":disabled")) return;
    close(); trigger.current?.focus();
    if (option.value !== String(value)) onChange({ target: { value: option.value } });
  };
  useLayoutEffect(() => {
    if (!open || !menu.current || !trigger.current) return;
    const popup = menu.current, button = trigger.current;
    const position = () => {
      const rect = button.getBoundingClientRect(), gap = 6, inset = 8;
      const below = innerHeight - rect.bottom - gap - inset, above = rect.top - gap - inset;
      const useAbove = below < 180 && above > below;
      popup.style.width = `${Math.min(Math.max(rect.width, 160), innerWidth - inset * 2)}px`;
      popup.style.maxHeight = `${Math.max(0, Math.min(320, useAbove ? above : below))}px`;
      popup.style.left = `${Math.max(inset, Math.min(rect.left, innerWidth - popup.getBoundingClientRect().width - inset))}px`;
      popup.style.top = `${useAbove ? Math.max(inset, rect.top - gap - popup.getBoundingClientRect().height) : rect.bottom + gap}px`;
    };
    popup.showPopover(); position();
    const outside = (event: PointerEvent) => { if (!popup.contains(event.target as Node) && !button.contains(event.target as Node)) close(); };
    const scroll = (event: Event) => { if (!popup.contains(event.target as Node)) position(); };
    document.addEventListener("pointerdown", outside, true);
    window.addEventListener("resize", close);
    document.addEventListener("scroll", scroll, true);
    return () => {
      popup.hidePopover(); document.removeEventListener("pointerdown", outside, true);
      window.removeEventListener("resize", close); document.removeEventListener("scroll", scroll, true);
    };
  }, [open]);
  useEffect(() => { if (open) menu.current?.querySelector(`[data-index="${active}"]`)?.scrollIntoView({ block: "nearest" }); }, [active, open]);
  useEffect(() => { if (disabled) close(); }, [disabled]);
  return <>
    <button {...props} ref={element => { trigger.current = element; if (typeof forwardedRef === "function") forwardedRef(element); else if (forwardedRef) forwardedRef.current = element; }}
      type="button" role="combobox" className={`select-trigger ${className}`} disabled={disabled}
      aria-haspopup="listbox" aria-expanded={open} aria-controls={listId} aria-activedescendant={open && active >= 0 ? `${uid}-${active}` : undefined}
      onBlur={close} onClick={() => { trigger.current?.focus(); if (open) close(); else reveal(); }}
      onKeyDown={event => {
        if (event.altKey || event.metaKey || event.ctrlKey) return;
        const key = event.key;
        if (["ArrowDown", "ArrowUp", "Home", "End"].includes(key)) {
          event.preventDefault(); event.stopPropagation();
          if (!open) { reveal(key === "Home" ? enabled[0] : key === "End" ? enabled.at(-1) : selected); return; }
          const index = enabled.indexOf(active);
          setActive(key === "Home" ? enabled[0] : key === "End" ? enabled[enabled.length - 1] : enabled[(index + (key === "ArrowDown" ? 1 : -1) + enabled.length) % enabled.length]);
        } else if (key === "Enter" || key === " ") {
          event.preventDefault(); event.stopPropagation(); if (open) choose(active); else reveal();
        } else if (key === "Escape" && open) {
          event.preventDefault(); event.stopPropagation(); close();
        } else if (key === "Tab") close();
        else if (key.length === 1) {
          event.preventDefault(); event.stopPropagation();
          const now = Date.now(); typed.current = { text: (now - typed.current.at < 700 ? typed.current.text : "") + key.toLowerCase(), at: now };
          const query = typed.current.text;
          const repeated = [...query].every(char => char === query[0]);
          const start = repeated ? active + 1 : Math.max(active, 0);
          const match = options.map((_, index) => (start + index) % options.length).find(index => !options[index].disabled && options[index].label.toLowerCase().startsWith(repeated ? query[0] : query));
          if (match !== undefined) { setActive(match); setOpen(true); }
        }
      }}><span className="select-value" title={options[selected]?.label}>{options[selected]?.label || "Choose…"}</span><ChevronDown aria-hidden="true" className="icon select-chevron" /></button>
    <div ref={menu} id={listId} className="select-menu" role="listbox" popover="manual" aria-label={props["aria-label"] || "Options"}
      onPointerDown={event => event.preventDefault()}>
      {options.map((option, index) => <div key={option.value} id={`${uid}-${index}`} data-index={index} role="option" aria-selected={index === selected} aria-disabled={option.disabled || undefined}
        className={`select-option ${active === index ? "active" : ""}`} onPointerMove={() => { if (!option.disabled) setActive(index); }} onClick={() => choose(index)}>
        <span>{option.label}</span>{index === selected && <Check className="icon" aria-hidden="true" />}
      </div>)}
    </div>
  </>;
});
