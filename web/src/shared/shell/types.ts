/** One bottom-bar tab: a screen the user can switch to. */
export interface TabDef {
  readonly id: string;
  readonly label: string;
  /** Inline SVG markup for the 24x24 icon (see icons.ts). */
  readonly icon: string;
}

/** The mounted shell: one pane per tab, and a way to switch. */
export interface TabShell {
  /** Where a feature mounts for each tab id. */
  readonly panes: Readonly<Record<string, HTMLElement>>;
  /** The right-hand end of the title bar, visible from every tab (the connection control mounts here). */
  readonly bar: HTMLElement;
  select(id: string): void;
}
