# Markdown Text Editor — Design Document

## Overview

A **modeless, nano-like** terminal Markdown editor designed as a reusable
`StatefulWidget` that can be embedded in larger TUI applications built with
[ratatui](https://ratatui.rs) and [crossterm](https://github.com/crossterm-rs/crossterm).

### Goals

- Edit small Markdown documents (1–2 printed pages) comfortably in a terminal.
- Provide a clean, familiar key-binding set similar to nano or simple GUI editors.
- Serve as an **embeddable component** — the host application drives the event
  loop and decides when to show the editor.
- Expose a programmatic API so the host can get/set content, register custom
  slash commands, and supply mention data.

### Non-Goals

- Large file support / syntax highlighting for non-Markdown languages.
- Multiple open files / tabs.
- Plugin system beyond the slash-command and mention extension points.

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                     │
│  (SQLite DB, file path management, AI agent dispatch)   │
├─────────────────────────────────────────────────────────┤
│                  Editor Shell / Chrome                   │
│  (status bar, help bar, key binding dispatch, dialogs)  │
├─────────────────────────────────────────────────────────┤
│                  Editor Widget (core)                    │
│  ┌──────────┬──────────┬──────────┬───────────────────┐ │
│  │  Buffer   │ Renderer │ Overlays │  Preview Pane     │ │
│  │  (ropey)  │ (syntax) │ (slash/  │  (pulldown-cmark) │ │
│  │           │          │  mention)│                   │ │
│  └──────────┴──────────┴──────────┴───────────────────┘ │
├─────────────────────────────────────────────────────────┤
│              Undo/Redo & Clipboard                      │
├─────────────────────────────────────────────────────────┤
│           ratatui + crossterm (TUI framework)           │
└─────────────────────────────────────────────────────────┘
```

The editor is deliberately layered so that each concern can be developed,
tested, and reasoned about in isolation.

---

## Module Structure

```
text-editor/
├── Cargo.toml
├── DESIGN.md
└── src/
    ├── main.rs                    # App entry point, event loop
    ├── app.rs                     # Application state
    ├── editor/
    │   ├── mod.rs                 # Public API: MarkdownEditor
    │   ├── buffer.rs              # Rope-based text buffer
    │   ├── cursor.rs              # Cursor position, selection ranges
    │   ├── history.rs             # Undo/redo stack
    │   ├── clipboard.rs           # System clipboard via arboard
    │   ├── input.rs               # Key/mouse event → editor action mapping
    │   ├── actions.rs             # Editor action enum and dispatch
    │   ├── viewport.rs            # Scroll state, visible region calculation
    │   ├── wrap.rs                # Visual word-wrap computation
    │   ├── highlight.rs           # Markdown syntax highlighting
    │   ├── preview.rs             # Markdown → rendered TUI preview
    │   ├── recovery.rs            # Auto-save / crash recovery
    │   ├── render.rs              # Widget rendering (StatefulWidget impl)
    │   └── overlay/
    │       ├── mod.rs             # Overlay trait and manager
    │       ├── slash_command.rs   # Slash command popup
    │       └── mention.rs         # @mention popup
    └── ui/
        ├── mod.rs                 # UI composition
        ├── status_bar.rs          # Bottom status bar
        └── help_bar.rs            # Top/bottom help hints
```

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `ratatui` | 0.29 | TUI widget framework (crossterm feature) |
| `crossterm` | 0.28 | Terminal input / output primitives |
| `ropey` | 1 | Efficient rope-based text buffer |
| `pulldown-cmark` | 0.12 | CommonMark parser for syntax highlighting and preview |
| `arboard` | 3 | Cross-platform system clipboard access |
| `unicode-width` | 0.2 | Accurate display-width calculation for CJK / emoji |

---

## Key Data Structures

### `Buffer` (`buffer.rs`)

Wraps a `ropey::Rope` and tracks additional metadata:

```rust
pub struct Buffer {
    rope: Rope,
    file_path: Option<PathBuf>,
    is_dirty: bool,
    saved_hash: u64,
}
```

Public methods:
- `insert_char(pos, ch)` / `insert_text(pos, text)`
- `delete_range(start, end)`
- `get_text() -> String`
- `set_text(text)`
- `save()` — writes to `file_path`
- `from_file(path)` — loads a file
- `from_template(template)` — initialises from a template string

### `Cursor` (`cursor.rs`)

```rust
pub struct Position {
    pub line: usize,  // 0-indexed logical line
    pub col: usize,   // 0-indexed character column
}

pub struct Cursor {
    pub pos: Position,
    pub anchor: Option<Position>,  // selection anchor; None = no selection
    pub preferred_col: usize,      // sticky column for up/down movement
}
```

Methods: `selection_range`, `has_selection`, `start_selection`,
`clear_selection`.

### `History` (`history.rs`)

```rust
pub enum EditKind {
    Insert,
    Delete,
    Replace,
}

pub struct Edit {
    pub kind: EditKind,
    pub cursor_before: Position,
    pub cursor_after: Position,
    pub timestamp: std::time::Instant,
}

pub struct History {
    undo_stack: Vec<EditGroup>,
    redo_stack: Vec<EditGroup>,
}
```

Rapid edits within a 300 ms window are coalesced into a single `EditGroup`.

### `EditorAction` (`actions.rs`)

A comprehensive enum covering every operation the editor can perform:

- Cursor movement (character, word, line, document)
- Selection (extend, select-all, clear)
- Editing (insert, delete, replace)
- Clipboard (copy, cut, paste)
- Undo / redo
- Markdown shortcuts (bold, italic, code, link)
- Overlay management (open slash-command, open mention, close)
- File operations (save, quit)
- View toggles (preview, line numbers)

### `MarkdownEditorState` (`render.rs`)

The complete runtime state held by the host application:

```rust
pub struct MarkdownEditorState {
    pub buffer: Buffer,
    pub cursor: Cursor,
    pub history: History,
    pub viewport: Viewport,
    pub highlight_cache: Vec<Vec<StyledRange>>,
    pub overlay: Option<Box<dyn Overlay>>,
    pub config: EditorConfig,
}
```

### `EditorConfig` (`render.rs`)

```rust
pub struct EditorConfig {
    pub tab_width: usize,
    pub show_status_bar: bool,
    pub show_help_hints: bool,
    pub show_line_numbers: bool,
    pub theme: MarkdownTheme,
}
```

---

## Key Bindings

| Key | Action |
|-----|--------|
| Arrow keys | Move cursor |
| Ctrl+Arrow | Move by word |
| Shift+Arrow | Extend selection |
| Home / End | Beginning / end of line |
| Ctrl+Home / Ctrl+End | Beginning / end of document |
| Ctrl+A | Select all |
| Ctrl+C | Copy |
| Ctrl+X | Cut |
| Ctrl+V | Paste |
| Ctrl+Z | Undo |
| Ctrl+Y | Redo |
| Ctrl+S | Save |
| Ctrl+B | Toggle bold |
| Ctrl+I | Toggle italic |
| Ctrl+\` | Toggle inline code |
| Ctrl+K | Insert link |
| Ctrl+P | Toggle preview pane |
| Ctrl+L | Toggle line numbers |
| Tab | Indent |
| Shift+Tab | Dedent |
| Ctrl+Q | Quit |
| `/` | Slash command menu (at line start or after space) |
| `@` | Mention picker (after space or at line start) |
| Mouse click | Position cursor |
| Mouse drag | Select text |
| Scroll wheel | Scroll viewport |

---

## Slash Commands

Triggered by typing `/` at the start of a line or after a space.  A popup
lists available commands; typing filters the list.

Built-in commands:

| Command | Description |
|---------|-------------|
| `/h1` | Heading 1 |
| `/h2` | Heading 2 |
| `/h3` | Heading 3 |
| `/bullet` | Bullet list item |
| `/numbered` | Numbered list item |
| `/todo` | Checkbox item |
| `/code` | Fenced code block |
| `/quote` | Block quote |
| `/divider` | Horizontal rule |
| `/table` | Markdown table scaffold |

The host application can register additional commands via
`MarkdownEditor::register_slash_command`.

---

## @Mentions

Triggered by typing `@` after a space or at the start of a line.  The host
must supply a `MentionProvider` implementation that resolves matching
`Mentionable` items.

```rust
pub trait MentionProvider: Send + Sync {
    fn query(&self, prefix: &str) -> Vec<Mentionable>;
}

pub struct Mentionable {
    pub id: String,
    pub display_name: String,
    pub kind: MentionKind,
    pub description: Option<String>,
}

pub enum MentionKind {
    User,
    AiAgent,
    Service,
}
```

---

## Auto-Save / Recovery

`RecoveryManager` writes a recovery snapshot to a configurable directory every
30 seconds (while the buffer is dirty).  On startup, `check_recovery()` detects
an existing recovery file and prompts the user to restore it.  After a
successful save, `clear_recovery()` removes the recovery file.

---

## Markdown Preview

When the preview pane is toggled on (`Ctrl+P`), the editor splits horizontally:
the left half shows the raw Markdown editor and the right half shows a rendered
preview produced by `render_preview()`, which walks the `pulldown-cmark` event
stream and produces styled `ratatui::text::Line` values.

---

## Implementation Phases

### Phase 1 — Core Editor (MVP)

Buffer, cursor, basic editing, word wrap, `StatefulWidget`, status bar,
save/quit, help hints.

### Phase 2 — Selection & Clipboard

Shift-selection, Ctrl+A, system clipboard integration, mouse support.

### Phase 3 — Undo/Redo & History

Edit history with 300 ms grouping, unlimited undo/redo, auto-save recovery,
dirty tracking.

### Phase 4 — Markdown Intelligence

Syntax highlighting, Markdown shortcuts (bold/italic/code/link), smart list
continuation, toggleable line numbers.

### Phase 5 — Preview & Overlays

Live preview pane, slash commands, @mentions.

### Phase 6 — Polish & Extensibility

Scrollbar, custom commands, resize handling, accessibility improvements,
potential extraction to a standalone crate.
