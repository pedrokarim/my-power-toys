# MyPowerToys — GUI Design System

Shared design language for all MyPowerToys tool windows built with egui/eframe. This file is the **single source of truth** — every module's `theme.rs` should derive from these tokens.

## Window Families

MyPowerToys has two families of GUI windows:

| Family | Use case | Background | Examples |
|--------|----------|------------|----------|
| **Opaque** | Standalone tool windows | Solid dark fills | Color Picker, Image Resizer |
| **Glassmorphism** | Floating overlays / HUDs | Semi-transparent with blur | Command Palette, Peek |

This document covers the **opaque** family. Glassmorphism overlays share the same text and accent colors but use `from_rgba_premultiplied` backgrounds with alpha channels.

---

## Color Palette

### Backgrounds

| Token | RGB | Hex | Usage |
|-------|-----|-----|-------|
| `BG_PRIMARY` | `(24, 24, 30)` | `#18181e` | Main window fill |
| `BG_SECONDARY` | `(32, 32, 40)` | `#202028` | Panels, progress bar track |
| `BG_CARD` | `(38, 38, 48)` | `#262630` | Content cards, format cards |
| `BG_HEADER` | `(20, 20, 26)` | `#14141a` | Header bar |
| `BG_HOVER` | `(50, 50, 62)` | `#32323e` | Interactive element hover |
| `BG_CHIP` | `(45, 45, 56)` | `#2d2d38` | Unselected chip / disabled button |
| `BG_CHIP_SELECTED` | `(55, 75, 140)` | `#374b8c` | Selected chip |
| `BG_BUTTON` | `(55, 75, 140)` | `#374b8c` | Primary action button |
| `BG_BUTTON_HOVER` | `(70, 95, 170)` | `#465faa` | Primary button hover |
| `BG_SUCCESS` | `(45, 140, 80)` | `#2d8c50` | Success state background |
| `BG_ERROR` | `(180, 60, 60)` | `#b43c3c` | Error state background |
| `BG_PROGRESS` | `(55, 75, 140)` | `#374b8c` | Progress bar fill |

### Text

| Token | RGB | Hex | Usage |
|-------|-----|-----|-------|
| `TEXT_PRIMARY` | `(235, 235, 240)` | `#ebebf0` | Headings, values, main content |
| `TEXT_SECONDARY` | `(150, 150, 168)` | `#9696a8` | Descriptions, labels |
| `TEXT_MUTED` | `(90, 90, 108)` | `#5a5a6c` | Disabled text, hints, placeholders |
| `TEXT_ACCENT` | `(130, 160, 240)` | `#82a0f0` | Links, section labels, interactive text |
| `TEXT_SUCCESS` | `(100, 210, 130)` | `#64d282` | Success messages |
| `TEXT_ERROR` | `(240, 100, 100)` | `#f06464` | Error messages |

### Borders & Separators

| Token | RGB | Hex | Usage |
|-------|-----|-----|-------|
| `SEPARATOR` | `(44, 44, 54)` | `#2c2c36` | Horizontal dividers |
| `CARD_BORDER` | `(50, 50, 62)` | `#32323e` | Card outlines |
| `DROP_ZONE_BORDER` | `(70, 90, 150)` | `#465a96` | Drop zone dashed border |
| `DROP_ZONE_BORDER_ACTIVE` | `(100, 130, 210)` | `#6482d2` | Drop zone active/dragging |

### Glassmorphism Variant

For overlay windows (Command Palette, Peek), backgrounds use alpha for transparency:

| Token | RGBA | Usage |
|-------|------|-------|
| `BG_PRIMARY` | `(25, 25, 35, 220-230)` | Main overlay fill |
| `BG_HEADER` | `(30, 30, 42, 240)` | Overlay header |
| `BG_SELECTED` | `(55, 75, 135, 180)` | Selected item |
| `BG_HOVER` | `(45, 45, 60, 160)` | Hover state |
| `SEPARATOR` | `(80, 80, 100, 60)` | Dividers |
| `BORDER` | `(90, 90, 120, 80)` | Outline border |
| `ACCENT` | `(110, 150, 240)` | Same as `TEXT_ACCENT` |

---

## Typography

All text uses egui's proportional font (system default). Monospace is used for code values and format badges.

| Token | Size (pt) | Usage |
|-------|-----------|-------|
| `FONT_TITLE` | 18 | Window title in header |
| `FONT_BUTTON` | 14 | Primary action buttons |
| `FONT_BODY` | 13 | General content, format values |
| `FONT_CHIP` | 12 | Chip labels |
| `FONT_SECTION` | 12 | Section labels (UPPERCASE) |
| `FONT_SMALL` | 11.5 | Secondary info, descriptions |
| `FONT_FILE` | 12 | File names in lists |
| `FONT_ICON` | 15–16 | Icon characters |

---

## Dimensions & Spacing

### Window

| Property | Value | Notes |
|----------|-------|-------|
| Min width | 340–400 px | Depends on content |
| Inner margin | 16–20 px | `INNER_MARGIN` / `INNER_PADDING` |
| Corner radius | 8–12 px | Window rounding |
| Centered | First 5 frames | Via `ViewportCommand::OuterPosition` |

### Components

| Component | Height | Radius | Notes |
|-----------|--------|--------|-------|
| Chip | 30 px | 6 px | Preset/format selectors |
| Card | 36–48 px | 8 px | File cards, format cards |
| Button | 38 px | 8 px | Full-width action button |
| Section spacing | 16 px | — | Between sections |
| Drop zone | 120 px | 8 px | File drop area |
| Accent bar | 3 px wide | — | Section label indicator |

---

## Components

### Section Label with Accent Bar

Each section starts with a colored vertical bar and an uppercase label:

```
│ SECTION NAME                [optional action]
```

- Accent bar: 3 px wide, `TEXT_ACCENT` color, full height of the label
- Label text: `FONT_SECTION`, `TEXT_ACCENT`, uppercase
- Optional right-aligned action text in `TEXT_ACCENT`

```rust
fn draw_section_label(ui: &mut egui::Ui, label: &str, right_text: Option<&str>) {
    let bar_rect = egui::Rect::from_min_size(pos, egui::vec2(3.0, height));
    painter.rect_filled(bar_rect, 1.0, TEXT_ACCENT);
    // ... label to the right of bar, right_text aligned to the right
}
```

### Chip Selector

Rounded toggle buttons for choosing between options (presets, formats, etc.):

```
[Option A]  [Option B]  [● Option C]  [Option D]
```

- Background: `BG_CHIP` (unselected) / `BG_CHIP_SELECTED` (selected)
- Border: 1 px `TEXT_ACCENT` when selected, none otherwise
- Text: `TEXT_MUTED` (unselected) / `TEXT_PRIMARY` (selected)
- Hover: `BG_HOVER` fill
- Size: `CHIP_HEIGHT` × dynamic width (measured from text)
- Radius: `CHIP_RADIUS` (6 px)

```rust
fn draw_chip_selector<T>(ui: &mut egui::Ui, options: &[T], current: &mut T)
where T: PartialEq + Display
{
    // Horizontal layout, wrap if needed
    // Each chip: allocate_exact_size → rect_filled → text galley
}
```

### Action Button

Full-width primary button for the main action:

```
┌──────────────────────────────────┐
│          ▶  Action Label         │
└──────────────────────────────────┘
```

- Fill: `BG_BUTTON` → `BG_BUTTON_HOVER` on hover
- Disabled: `BG_CHIP` fill, `TEXT_MUTED` text
- Text: `FONT_BUTTON`, `TEXT_PRIMARY`, centered
- Height: `BUTTON_HEIGHT` (38 px)
- Radius: `BUTTON_RADIUS` (8 px)

### Card

Rounded rectangle container for content items:

- Fill: `BG_CARD`
- Border: 1 px `CARD_BORDER`
- Radius: `CARD_RADIUS` (8 px)
- Optional left accent bar (4 px, colored by context)

### Format Card (Color Picker specific)

Displays a color format value with copy button:

```
┌─[■]──────────────────────────────────────┐
│  LABEL    value text              [copy]  │
└──────────────────────────────────────────┘
```

- Left accent bar: 4 px, current color
- Label: `FONT_SMALL` (11 pt), `TEXT_SECONDARY`
- Value: `FONT_BODY` (13 pt), `TEXT_PRIMARY`, monospace
- Copy button: 28 px, shows ✓ for 1 second after click

### Drop Zone

Dashed-border area for file drag & drop:

```
┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐
│             ↓                     │
│       Drop files here             │
│    PNG, JPEG, WebP, ...           │
└ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘
```

- Border: `DROP_ZONE_BORDER`, dashed (4 px segments, 4 px gap)
- Active (dragging): `DROP_ZONE_BORDER_ACTIVE`
- Arrow icon: drawn with `painter.line_segment()` — not Unicode
- Text: `TEXT_SECONDARY` (main), `TEXT_MUTED` (format hint)
- Height: `DROP_ZONE_HEIGHT` (120 px)

### Progress Bar

Replaces the action button during processing:

```
┌──────████████░░░░░░░░░░░░░░░░░░──┐
│        Processing... 3/10         │
└──────────────────────────────────┘
```

- Track: `BG_SECONDARY`
- Fill: `BG_PROGRESS`, width = `progress * track_width`
- Text: centered, `TEXT_PRIMARY`, `FONT_BODY`
- Radius: `BUTTON_RADIUS`

### Results Card

Post-action success/error summary:

- Success: `BG_SUCCESS` left accent, `TEXT_SUCCESS` message
- Error: `BG_ERROR` left accent, `TEXT_ERROR` message + details
- "Do more" button to reset state

---

## Custom Icons

**No emoji or icon fonts.** All icons are drawn programmatically with the egui `Painter` API. This avoids missing glyph issues across platforms.

### Drawing patterns

- **Line icons**: `painter.line_segment()` with `Stroke::new(width, color)`
- **Filled shapes**: `painter.rect_filled()`, `painter.circle_filled()`
- **Compound icons**: Combine multiple primitives (lines, rects, circles)

### Icon catalog

| Icon | Technique | Where used |
|------|-----------|------------|
| Header tool icon | 2 overlapping `rect_stroke` + diagonal arrow line | Image Resizer header |
| Download arrow | Vertical line + chevron + horizontal tray | Drop zone |
| Pencil (✏) | Unicode `\u{270F}` in text | Color Picker "Pick color" button |
| Gear (⚙) | Unicode `\u{2699}` in text | Color Picker settings |
| Checkmark | `line_segment` V-shape | Copy feedback, success state |
| Cross (✕) | `line_segment` X-shape | Remove button, error state |

> **Rule**: For icons larger than 14px or decorative elements, prefer painter-drawn shapes. For small inline icons in text (⚙, ✏), basic Unicode characters in the BMP range are acceptable — they are included in egui's default font.

---

## Visual Setup

Every tool window initializes its visuals the same way via `setup_visuals()`:

```rust
fn setup_visuals(ctx: &egui::Context) {
    let mut vis = egui::Visuals::dark();
    vis.window_fill = BG_PRIMARY;
    vis.panel_fill = BG_PRIMARY;
    vis.window_shadow = egui::epaint::Shadow::NONE;
    vis.window_stroke = egui::Stroke::NONE;
    vis.widgets.noninteractive.bg_fill = BG_PRIMARY;
    vis.widgets.inactive.bg_fill = BG_SECONDARY;
    vis.widgets.hovered.bg_fill = BG_HOVER;
    vis.widgets.active.bg_fill = BG_BUTTON;
    ctx.set_visuals(vis);
}
```

### Window centering

Center the window on screen during the first 5 frames:

```rust
if self.frame_count < 5 {
    if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
        let x = (monitor.x - WINDOW_WIDTH) / 2.0;
        let y = (monitor.y - WINDOW_HEIGHT) / 2.0;
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
            egui::pos2(x, y),
        ));
    }
    self.frame_count += 1;
}
```

### Window options (eframe::NativeOptions)

```rust
let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
        .with_min_inner_size([MIN_WIDTH, MIN_HEIGHT])
        .with_title("Tool Name")
        .with_resizable(true),
    ..Default::default()
};
```

---

## Layout Patterns

### Standard Tool Window

```
┌──────────────────────────────────┐
│  [icon]  Tool Name               │  ← Header (BG_HEADER)
│  Short description               │
├──────────────────────────────────┤
│                                  │
│  | SECTION 1            [action] │  ← Section label
│  ┌────────────────────────────┐  │
│  │        Content area        │  │  ← Cards / interactive
│  └────────────────────────────┘  │
│                                  │
│  ─────────────────────────────   │  ← Separator
│                                  │
│  | SECTION 2                     │
│  [Chip A] [Chip B] [● Chip C]   │  ← Chip selector
│                                  │
│  ┌────────────────────────────┐  │
│  │      ▶  Primary Action    │  │  ← Action button
│  └────────────────────────────┘  │
└──────────────────────────────────┘
```

### Header

- Background: `BG_HEADER`
- Left: custom-drawn icon (painter API)
- Title: `FONT_TITLE`, `TEXT_PRIMARY`, bold
- Subtitle: `FONT_SMALL`, `TEXT_SECONDARY`

---

## Applying to a New Module

1. Copy `theme.rs` from an existing module (Image Resizer is the most complete)
2. Adjust window-specific dimensions (`WINDOW_WIDTH`, `WINDOW_HEIGHT`)
3. Add module-specific tokens if needed (e.g., `SHADE_BAR_HEIGHT` for Color Picker)
4. Keep all shared color tokens identical — do not drift
5. Implement `setup_visuals()` using the shared pattern
6. Draw custom icons with the painter API — no emoji above U+2800
7. Use `draw_section_label`, `draw_chip_selector`, `draw_action_button` patterns

---

## File Structure

Each GUI module follows this structure:

```
crates/<module>/src/
├── lib.rs              # PowerModule trait + binary spawner
├── main.rs             # GUI binary entry point (lock file, config, launch)
└── gui/
    ├── mod.rs          # pub mod theme; pub mod window;
    ├── theme.rs        # Color/dimension tokens from this design system
    └── window.rs       # Main egui app (all UI rendering)
```
