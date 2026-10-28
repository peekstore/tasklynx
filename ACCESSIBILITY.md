# Accessibility

Tasklynx aims to be usable by everyone, including people who rely on assistive technology, keyboard-only navigation, high-contrast themes, or reduced motion.

## Current state

Tasklynx ships three interfaces with different accessibility characteristics.

### CLI

The command-line interface is the most accessible surface. It uses plain text in and out, requires no pointer, and works with screen readers, braille displays, and terminal automation.

- Run `tx <command> --help` for usage text.
- Output is line-oriented and suitable for piping to other tools.
- Exit status is non-zero on error.

### Terminal board (`tx board`)

The Kanban TUI uses `ratatui` and draws with box characters and color.

- Navigation is keyboard-only.
- Color is supplemented with column headings and per-column layout, so status is not conveyed by color alone.
- Reduced-motion and high-contrast support depend on your terminal emulator. We recommend configuring your terminal's accessibility options (for example, a high-contrast palette and screen-reader cursor tracking).

### Web dashboard (`tx web`)

The web UI is a single embedded page.

- It is keyboard navigable and uses semantic HTML for the task list.
- Theme colors are configurable in `config.toml` so you can raise contrast.
- It does not currently ship a dedicated screen-reader mode or a documented WCAG conformance level.

## Design commitments

When contributing to Tasklynx, please:

- Do not encode meaning in color alone; pair it with text or position.
- Keep the CLI output readable by screen readers and line-based tools.
- Preserve keyboard access for every interactive action.
- Avoid gratuitous animation; respect the user's terminal or browser settings where practical.
- Use plain language in user-facing messages and documentation.
- Ensure documentation images have descriptive alt text.

## Known limitations

- No formal accessibility audit or WCAG certification has been performed.
- The TUI's visual layout cannot be fully represented to screen readers.
- The web dashboard does not yet support a user-selectable high-contrast theme or reduced-motion preference inside the app.
- Documentation screenshots are raster images; text in them may not be accessible.

## Feedback

If you encounter an accessibility barrier, please open an issue at [github.com/peekstore/tasklynx/issues](https://github.com/peekstore/tasklynx/issues) describing:

- the interface you were using (CLI, board, or web),
- the assistive technology or settings involved,
- the barrier you hit and what would have helped.

You can also email [peekstore585@gmail.com](mailto:peekstore585@gmail.com). We treat accessibility reports as bugs.

## References

- [WAI-ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)
- [Web Content Accessibility Guidelines (WCAG) 2.2](https://www.w3.org/TR/WCAG22/)
- [Terminal accessibility guidance](https://github.com/rothgar/awesome-tuis#accessibility)
