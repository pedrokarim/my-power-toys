"""Nautilus extension for MyPowerToys Bulk Rename.

Install to: ~/.local/share/nautilus-python/extensions/
Requires: python3-nautilus (nautilus-python)
"""

import os
import subprocess
from gi.repository import Nautilus, GObject


class BulkRenameExtension(GObject.GObject, Nautilus.MenuProvider):
    def get_file_items(self, *args):
        # nautilus-python 4.x passes (files,), 3.x passes (window, files)
        files = args[-1]
        if not files:
            return []

        item = Nautilus.MenuItem(
            name="MyPowerToys::BulkRename",
            label="Bulk Rename with MyPowerToys",
            tip="Rename selected files using patterns with live preview",
        )
        item.connect("activate", self._on_activate, files)
        return [item]

    def get_background_items(self, *args):
        # Right-click on folder background
        file = args[-1] if len(args) == 1 else args[-1]
        if isinstance(file, list):
            return []

        if not file.is_directory():
            return []

        item = Nautilus.MenuItem(
            name="MyPowerToys::BulkRenameFolder",
            label="Bulk Rename with MyPowerToys",
            tip="Rename files in this folder using patterns",
        )
        item.connect("activate", self._on_activate_folder, file)
        return [item]

    def _on_activate(self, _menu, files):
        paths = [f.get_location().get_path() for f in files if f.get_location()]
        paths = [p for p in paths if p]
        if paths:
            subprocess.Popen(["mpt-bulk-rename"] + paths)

    def _on_activate_folder(self, _menu, folder):
        path = folder.get_location().get_path()
        if path:
            subprocess.Popen(["mpt-bulk-rename", path])
