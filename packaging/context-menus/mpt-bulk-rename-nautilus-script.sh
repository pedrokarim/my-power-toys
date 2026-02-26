#!/bin/bash
# Nautilus script fallback for MyPowerToys Bulk Rename.
# Install to: ~/.local/share/nautilus/scripts/Bulk Rename (MyPowerToys)
#
# Uses the NAUTILUS_SCRIPT_SELECTED_FILE_PATHS environment variable
# which Nautilus sets to the newline-separated list of selected file paths.

if [ -z "$NAUTILUS_SCRIPT_SELECTED_FILE_PATHS" ]; then
    exit 0
fi

# Build argument array from newline-separated paths
args=()
while IFS= read -r path; do
    [ -n "$path" ] && args+=("$path")
done <<< "$NAUTILUS_SCRIPT_SELECTED_FILE_PATHS"

if [ ${#args[@]} -gt 0 ]; then
    exec mpt-bulk-rename "${args[@]}"
fi
