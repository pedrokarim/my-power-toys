# Screenshot Methodology for GUI Binaries

How to capture clean, consistent screenshots of MyPowerToys GUI modules for the documentation pages.

## Prerequisites

```bash
sudo apt install xdotool imagemagick scrot
```

- **xdotool** — find windows by name/class
- **ImageMagick** (`import`) — capture a specific window by ID
- **scrot** — fallback full-screen/window capture

## General Workflow

1. **Launch the binary** with the right state/arguments
2. **Wait** for the window to appear (sleep or `xdotool --sync`)
3. **Find the window ID** using `xdotool search --name`
4. **Capture** with `import -window $WID output.png`
5. **Kill the process** to clean up

## Per-Module Instructions

### Peek (`mpt-peek`)

Peek takes a file path as CLI argument, which makes it easy to script.

```bash
# Remove stale lock file
rm -f /tmp/mpt-peek.lock

# Text file preview
mpt-peek /path/to/some/source-file.rs &
sleep 1
WID=$(xdotool search --name "Peek" | head -1)
import -window "$WID" docs/images/peek-text.png
kill %1

# Image preview
rm -f /tmp/mpt-peek.lock
mpt-peek /path/to/some/image.png &
sleep 1
WID=$(xdotool search --name "Peek" | head -1)
import -window "$WID" docs/images/peek-image.png
kill %1

# Directory preview
rm -f /tmp/mpt-peek.lock
mpt-peek /path/to/some/directory/ &
sleep 1
WID=$(xdotool search --name "Peek" | head -1)
import -window "$WID" docs/images/peek-directory.png
kill %1
```

### Color Picker (`mpt-color-picker`)

Color Picker has three modes configured in `~/.config/my-power-toys/config.toml`:
- `pick-and-edit` — pick a color then open editor (default)
- `pick-and-close` — pick a color, copy to clipboard, close
- `editor-only` — open the editor directly (best for screenshots)

```bash
# Temporarily set editor-only mode for a clean screenshot
# Edit ~/.config/my-power-toys/config.toml:
#   [color-picker]
#   behavior = "editor-only"

mpt-color-picker &
sleep 1
WID=$(xdotool search --name "Color Picker" | head -1)
import -window "$WID" docs/images/color-picker-editor.png
kill %1

# Restore original behavior after capture:
#   behavior = "pick-and-edit"
```

> **Tip**: To capture the magnifying-glass overlay, use `pick-and-edit` mode and
> take a full-screen screenshot with `scrot docs/images/color-picker-overlay.png`
> while the overlay is active. You have ~2 seconds before the pick.

### Image Resizer (`mpt-image-resizer`)

Image Resizer opens directly with no arguments needed.

```bash
rm -f /tmp/mpt-image-resizer.lock
mpt-image-resizer &
sleep 2
WID=$(xdotool search --name "Image Resizer" | head -1)
import -window "$WID" docs/images/image-resizer.png
kill %1
```

### Bulk Rename (`mpt-bulk-rename`)

Bulk Rename works best with file arguments. Create a temp directory with sample files.

```bash
# Create sample files
mkdir -p /tmp/bulk-rename-demo
touch /tmp/bulk-rename-demo/photo_{001..010}.jpg
touch /tmp/bulk-rename-demo/document_{a..e}.txt

rm -f /tmp/mpt-bulk-rename.lock
mpt-bulk-rename /tmp/bulk-rename-demo/* &
sleep 2
WID=$(xdotool search --name "Bulk Rename" | head -1)
import -window "$WID" docs/images/bulk-rename.png
kill %1

# Clean up
rm -rf /tmp/bulk-rename-demo
```

### Command Palette (`mpt-command-palette`)

The palette opens empty and needs keyboard input to show results.

```bash
# Empty search bar
mpt-command-palette &
sleep 1
WID=$(xdotool search --name "mpt-command-palette" | head -1)
import -window "$WID" docs/images/command-palette.png
kill %1

# With search results: type a query using xdotool
rm -f /tmp/mpt-command-palette.lock
mpt-command-palette &
sleep 1
WID=$(xdotool search --name "mpt-command-palette" | head -1)
xdotool type --window "$WID" --delay 50 "set"
sleep 0.5
import -window "$WID" docs/images/command-palette-results.png
kill %1
```

## Tips

- **Lock files**: GUI binaries use singleton lock files in `/tmp/mpt-<name>.lock`.
  Remove stale locks before re-launching: `rm -f /tmp/mpt-<name>.lock`

- **Window names**: Use `xdotool search --name` to find the window. If the name
  doesn't match, try `xdotool search --class` or check the window title with
  `xdotool getactivewindow getwindowname`.

- **Timing**: `sleep 1` is usually enough for the window to render. For slower
  machines or Wayland compositors, increase to `sleep 2`.

- **Wayland**: `xdotool` and `import` work under XWayland. If the app uses native
  Wayland, use `grim` + `slurp` instead:
  ```bash
  grim -g "$(slurp)" docs/images/screenshot.png
  ```

- **Consistent sizing**: The modules use fixed window sizes defined in their
  `gui/theme.rs`. No resizing needed.

- **Output directory**: All screenshots go to `docs/images/` with descriptive
  names matching the HTML `<img src>` references.

## Current Screenshots

| File | Module | Description |
|------|--------|-------------|
| `peek-text.png` | Peek | Source code preview (Cargo.toml) |
| `peek-image.png` | Peek | Image preview (logo with Tux) |
| `peek-directory.png` | Peek | Directory listing (crates/) |
| `color-picker-editor.png` | Color Picker | Editor with HEX/RGB/HSL/HSV/CMYK |
| `image-resizer.png` | Image Resizer | Main window with drop zone and presets |
| `command-palette.png` | Command Palette | Empty search bar |
| `command-palette-results.png` | Command Palette | Search "set" with 8 results |
| `always-on-top-illustration.svg` | Always on Top | SVG illustration: pinned window with blue border above other windows |
| `awake-illustration.svg` | Awake | SVG illustration: coffee cup with four mode icons (indefinite, timed, expirable, screen on) |
| `bulk-rename.png` | Bulk Rename | Main window with search/replace, options, file table preview |
| `hosts-editor.png` | Hosts Editor | Main window with toggle, filter, entry table, add/edit form |
