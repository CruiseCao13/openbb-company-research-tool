import { useEffect, useMemo, useRef, useState } from "react";

export type CommandItem = {
  detail: string;
  disabled?: boolean;
  id: string;
  label: string;
  onRun: () => void;
};

export function CommandMenu({
  commands,
  emptyLabel,
  inputLabel,
  onClose,
  onQueryChange,
  open,
  placeholder,
  query,
  title
}: {
  commands: CommandItem[];
  emptyLabel: string;
  inputLabel: string;
  onClose: () => void;
  onQueryChange: (query: string) => void;
  open: boolean;
  placeholder: string;
  query: string;
  title: string;
}): JSX.Element | null {
  const inputRef = useRef<HTMLInputElement | null>(null);
  const [activeIndex, setActiveIndex] = useState(0);
  const filteredCommands = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    if (!normalized) {
      return commands;
    }
    return commands.filter((command) => `${command.label} ${command.detail}`.toLowerCase().includes(normalized));
  }, [commands, query]);

  useEffect(() => {
    if (!open) {
      return;
    }
    setActiveIndex(0);
    window.setTimeout(() => inputRef.current?.focus(), 30);
  }, [open]);

  useEffect(() => {
    if (!open) {
      return;
    }

    function handleKeyDown(event: KeyboardEvent): void {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
        return;
      }
      if (event.key === "ArrowDown") {
        event.preventDefault();
        setActiveIndex((index) => Math.min(filteredCommands.length - 1, index + 1));
        return;
      }
      if (event.key === "ArrowUp") {
        event.preventDefault();
        setActiveIndex((index) => Math.max(0, index - 1));
        return;
      }
      if (event.key === "Enter") {
        event.preventDefault();
        const command = filteredCommands[activeIndex];
        if (command && !command.disabled) {
          command.onRun();
          onClose();
        }
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [activeIndex, filteredCommands, onClose, open]);

  if (!open) {
    return null;
  }

  return (
    <div className="command-menu-backdrop" role="presentation">
      <section className="command-menu" aria-label={title} role="dialog">
        <div className="command-menu__header">
          <span className="mono-label">{title}</span>
          <button className="glass-pill command-menu__close" onClick={onClose} type="button">
            Esc
          </button>
        </div>
        <label className="command-menu__search">
          <span className="mono-label">{inputLabel}</span>
          <input
            autoComplete="off"
            onChange={(event) => onQueryChange(event.target.value)}
            placeholder={placeholder}
            ref={inputRef}
            value={query}
          />
        </label>
        <div className="command-menu__results" role="listbox">
          {filteredCommands.length === 0 ? (
            <div className="command-menu__empty">{emptyLabel}</div>
          ) : (
            filteredCommands.map((command, index) => (
              <button
                aria-selected={index === activeIndex}
                className={`command-menu__item${index === activeIndex ? " command-menu__item--active" : ""}`}
                disabled={command.disabled}
                key={command.id}
                onClick={() => {
                  if (!command.disabled) {
                    command.onRun();
                    onClose();
                  }
                }}
                onMouseEnter={() => setActiveIndex(index)}
                role="option"
                type="button"
              >
                <strong>{command.label}</strong>
                <span>{command.detail}</span>
              </button>
            ))
          )}
        </div>
      </section>
    </div>
  );
}
