# MyPowerToys - PowerToys for Linux

## Vision

Suite d'utilitaires systeme pour Linux, inspiree de Microsoft PowerToys, ecrite en Rust.
Application modulaire avec un daemon central et des modules independants activables/desactivables.

---

## Architecture Globale

```
my-power-toys/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── daemon/                 # Daemon central (tray icon, gestion modules)
│   ├── common/                 # Types partages, config, IPC, utils
│   ├── ui/                     # Interface settings (GTK4 ou egui)
│   ├── fancy-zones/            # Window tiling/zones
│   ├── app-launcher/           # Lanceur d'applications (PowerToys Run)
│   ├── color-picker/           # Pipette couleur ecran
│   ├── key-manager/            # Remappage de touches
│   ├── bulk-rename/            # Renommage en masse
│   ├── image-resizer/          # Redimensionnement d'images en masse
│   ├── screen-ruler/           # Mesure de pixels a l'ecran
│   ├── always-on-top/          # Epingler une fenetre au premier plan
│   ├── awake/                  # Empecher la mise en veille
│   ├── text-extractor/         # OCR depuis l'ecran
│   ├── mouse-utils/            # Utilitaires souris (find my mouse, crosshair)
│   ├── shortcut-guide/         # Overlay des raccourcis actifs
│   ├── paste-plain/            # Coller en texte brut
│   ├── peek/                   # Apercu rapide de fichiers (Quick Look)
│   └── hosts-editor/           # Editeur /etc/hosts graphique
└── assets/                     # Icones, themes, ressources
```

### Principes

- **Modulaire** : chaque outil = un crate independant, activable/desactivable
- **Leger** : daemon minimal en memoire, modules charges a la demande
- **Natif** : integration Wayland + X11 (priorite Wayland)
- **Configurable** : fichier TOML par module + UI settings globale
- **IPC** : communication daemon <-> modules via D-Bus ou Unix sockets

---

## Stack Technique

| Composant | Technologie |
|---|---|
| Langage | Rust (edition 2024) |
| Build | Cargo workspace |
| UI Fenetres | `iced` (settings, launcher, editors — Elm architecture, pur Rust) |
| UI Overlays | `egui`/`eframe` (color picker, ruler, zones — immediate mode, temps reel) |
| Tray Icon | `ksni` (StatusNotifierItem / systemd tray) |
| Hotkeys globaux | `evdev` + `uinput` (Wayland) / `x11rb` (X11) |
| Window management | `wayland-client` + wlr-protocols / `x11rb` |
| IPC | `zbus` (D-Bus) |
| Config | `serde` + `toml` |
| OCR | `leptess` (binding Tesseract) |
| Image processing | `image` crate |
| File watching | `notify` crate |
| Logging | `tracing` |
| Tests | `cargo test` + `insta` (snapshot tests) |
| CI | GitHub Actions |
| Packaging | `.deb`, `.rpm`, Flatpak, AUR |

---

## Modules - Detail

### 1. Daemon Central (`daemon`)
Le coeur du systeme. Tourne en arriere-plan.

**Responsabilites :**
- Icone tray avec menu (activer/desactiver modules, ouvrir settings, quitter)
- Charger/decharger les modules selon la config
- Gerer les raccourcis globaux (dispatcher vers le bon module)
- Autostart via fichier `.desktop` dans `~/.config/autostart/`

**Depends on :** `common`, `zbus`, `ksni`

---

### 2. Common (`common`)
Librairie partagee entre tous les modules.

**Contenu :**
- Trait `PowerModule` que chaque module implemente
- Systeme de config (load/save TOML depuis `~/.config/my-power-toys/`)
- Client/serveur IPC (D-Bus interfaces)
- Detection Wayland vs X11
- Helpers pour hotkeys, notifications desktop

**Trait principal :**
```rust
pub trait PowerModule: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn default_hotkey(&self) -> Option<Hotkey>;
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn is_running(&self) -> bool;
    fn on_hotkey(&mut self, hotkey: &Hotkey) -> Result<()>;
    fn config_schema(&self) -> ConfigSchema;
}
```

---

### 3. UI Settings (`ui`)
Interface graphique pour configurer tous les modules.

**Features :**
- Liste des modules avec toggle on/off
- Page de settings par module
- Configuration des raccourcis globaux
- Theme clair/sombre (suit le theme systeme)
- Recherche dans les settings

---

### 4. FancyZones (`fancy-zones`)
Gestion avancee du tiling de fenetres avec des zones predefinies.

**Features :**
- Editeur de layouts (grille, colonnes, custom)
- Drag & drop de fenetres dans les zones (overlay pendant le drag)
- Layouts par ecran / workspace
- Raccourci pour activer l'editeur de zones
- Snap avec raccourcis clavier (Super+fleches ameliore)
- Sauvegarde des positions par application

**Complexite : HAUTE**
- Necessite `wlr-foreign-toplevel-management` (Wayland) ou `_NET_WM` (X11)
- Overlay transparent pour afficher les zones

---

### 5. App Launcher (`app-launcher`)
Lanceur d'applications rapide style Spotlight/Alfred.

**Features :**
- Activation par raccourci (defaut: `Super+Space` ou `Alt+Space`)
- Recherche d'applications (.desktop files)
- Recherche de fichiers (indexation basique)
- Calculatrice integree (expressions math)
- Plugins : conversions d'unites, recherche web, commandes shell
- Historique et frecency (frequence + recency) pour le tri
- Interface minimaliste centree a l'ecran

**Complexite : HAUTE**
- Fenetre overlay Wayland/X11
- Indexation rapide des applications et fichiers

---

### 6. Color Picker (`color-picker`)
Pipette pour capturer une couleur a l'ecran.

**Features :**
- Activation par raccourci (defaut: `Super+Shift+C`)
- Loupe/zoom autour du curseur
- Affichage temps reel de la couleur sous le curseur
- Copie dans le presse-papier (HEX, RGB, HSL configurable)
- Historique des couleurs capturees
- Apercu de la couleur dans une petite popup

**Complexite : MOYENNE**
- Screenshot region via `xdg-desktop-portal` (Wayland) ou `XGetImage` (X11)

---

### 7. Key Manager (`key-manager`)
Remappage de touches et creation de raccourcis custom.

**Features :**
- Remapper une touche vers une autre
- Creer des raccourcis vers des actions (lancer app, commande, etc.)
- Remappage conditionnel (par application)
- Interface visuelle du clavier
- Import/export de la config

**Complexite : HAUTE**
- Interception bas-niveau via `evdev` + `uinput`
- Necessite permissions (groupe `input` ou capabilities)

---

### 8. Bulk Rename (`bulk-rename`)
Renommage de fichiers en masse avec regex et preview.

**Features :**
- Rechercher/remplacer avec regex
- Preview en temps reel des changements
- Enumeration (sequence de nombres)
- Changement de casse (upper, lower, title, camelCase)
- Modification des dates dans les noms
- Undo (historique des operations)
- Integration avec le file manager (menu contextuel via D-Bus)

**Complexite : BASSE-MOYENNE**

---

### 9. Image Resizer (`image-resizer`)
Redimensionnement d'images en masse.

**Features :**
- Presets de tailles (small, medium, large, phone, custom)
- Choix du format de sortie (PNG, JPEG, WebP)
- Qualite configurable
- Renommage automatique (suffixe ou dossier separe)
- Drag & drop ou selection de fichiers
- Integration file manager (menu contextuel)

**Complexite : BASSE**

---

### 10. Screen Ruler (`screen-ruler`)
Mesure de distances en pixels a l'ecran.

**Features :**
- Mode horizontal / vertical / rectangle / croix
- Affichage en pixels (et optionnel cm/inches selon DPI)
- Overlay semi-transparent
- Copie de la mesure dans le presse-papier

**Complexite : MOYENNE**

---

### 11. Always on Top (`always-on-top`)
Epingler une fenetre au premier plan.

**Features :**
- Raccourci pour toggle (defaut: `Super+T`)
- Bordure coloree sur la fenetre epinglee
- Indicateur dans la barre de titre ou overlay

**Complexite : BASSE**
- `wlr-foreign-toplevel` (Wayland) ou `_NET_WM_STATE_ABOVE` (X11)

---

### 12. Awake (`awake`)
Empecher la mise en veille de l'ecran.

**Features :**
- Toggle depuis le tray
- Mode temporise (30min, 1h, 2h, indefini)
- Mode "garder eveille tant que cette app tourne"
- Indicateur visuel dans le tray (icone differente)

**Complexite : BASSE**
- `systemd-inhibit` ou D-Bus `org.freedesktop.ScreenSaver.Inhibit`

---

### 13. Text Extractor (`text-extractor`)
OCR : capturer du texte depuis n'importe quelle zone de l'ecran.

**Features :**
- Raccourci pour activer (defaut: `Super+Shift+T`)
- Selection de zone a l'ecran
- Reconnaissance du texte (OCR via Tesseract)
- Copie automatique dans le presse-papier
- Support multi-langues

**Complexite : MOYENNE**
- Depend de `tesseract` installe sur le systeme

---

### 14. Mouse Utilities (`mouse-utils`)
Ameliorations pour la souris.

**Features :**
- **Find My Mouse** : double-clic Ctrl pour spotlight sur le curseur
- **Mouse Highlighter** : cercle colore autour du curseur lors des clics
- **Crosshair** : reticule permanent centre sur le curseur
- Configurable (couleurs, tailles, opacite, animation)

**Complexite : MOYENNE**
- Overlay qui suit le curseur en temps reel

---

### 15. Shortcut Guide (`shortcut-guide`)
Overlay affichant les raccourcis disponibles.

**Features :**
- Activation en maintenant `Super` pendant 1 seconde
- Affiche les raccourcis du DE actuel (GNOME, KDE, etc.)
- Affiche les raccourcis custom de MyPowerToys
- Layout visuel style cheatsheet

**Complexite : MOYENNE**

---

### 16. Paste as Plain Text (`paste-plain`)
Coller du texte sans formatage.

**Features :**
- Raccourci (defaut: `Super+Ctrl+V`)
- Intercepte le presse-papier, retire le formatage, colle en texte brut
- Option pour toujours coller en texte brut dans certaines apps

**Complexite : BASSE**
- Manipulation du clipboard via `wl-clipboard` ou `xclip`

---

### 17. Peek (`peek`)
Apercu rapide de fichiers sans ouvrir une application complete.

**Features :**
- Raccourci (defaut: `Space` dans le file manager, ou raccourci global)
- Preview : images, PDF, texte, code (avec coloration syntaxique), markdown, videos
- Fenetre popup legere, fermeture rapide avec Escape
- Navigation entre fichiers avec les fleches

**Complexite : MOYENNE-HAUTE**

---

### 18. Hosts Editor (`hosts-editor`)
Editeur graphique pour `/etc/hosts`.

**Features :**
- Liste des entrees avec toggle on/off (commente/decommente la ligne)
- Ajout/suppression/edition d'entrees
- Sauvegarde avec elevation de privileges (`pkexec`)
- Backup automatique avant modification
- Filtrage/recherche

**Complexite : BASSE**

---

## Phases de Developpement

### Phase 1 - Fondations (Semaines 1-3)
- [ ] Setup du workspace Cargo
- [ ] Crate `common` : trait `PowerModule`, config TOML, detection Wayland/X11
- [ ] Crate `daemon` : tray icon, chargement de modules, autostart
- [ ] IPC via D-Bus (`zbus`)
- [ ] Premier module simple pour valider l'archi : **Always on Top**
- [ ] CI basique (build + clippy + tests)

### Phase 2 - Modules essentiels (Semaines 4-8)
- [ ] **Awake** (simple, valide l'inhibition screensaver)
- [ ] **Paste as Plain Text** (simple, valide la capture de hotkeys)
- [ ] **Color Picker** (moyen, valide la capture d'ecran + overlay)
- [ ] **Hosts Editor** (simple, valide l'elevation de privileges)
- [ ] **Bulk Rename** (moyen, utilitaire standalone)
- [ ] **Image Resizer** (simple, utilitaire standalone)

### Phase 3 - Modules avances (Semaines 9-16)
- [ ] **Key Manager** (complexe, interception evdev)
- [ ] **Mouse Utilities** (moyen, overlay temps reel)
- [ ] **Screen Ruler** (moyen, overlay interactif)
- [ ] **Text Extractor** (moyen, OCR)
- [ ] **Shortcut Guide** (moyen, overlay informatif)

### Phase 4 - Modules complexes (Semaines 17-24)
- [ ] **App Launcher** (complexe, fenetre overlay + indexation)
- [ ] **FancyZones** (complexe, window management)
- [ ] **Peek** (moyen-complexe, previews multiformats)

### Phase 5 - Polish (Semaines 25+)
- [ ] UI Settings complete
- [ ] Packaging (deb, rpm, flatpak, AUR)
- [ ] Documentation utilisateur
- [ ] Site web / landing page
- [ ] Theming et personnalisation avancee

---

## Challenges Techniques Linux

| Challenge | Solution |
|---|---|
| Wayland ne permet pas la capture globale de touches | `evdev` + `uinput` (necessite privileges) |
| Wayland ne permet pas de positionner les fenetres | Protocols wlr (`wlr-layer-shell`, `wlr-foreign-toplevel`) |
| Wayland ne permet pas les screenshots | `xdg-desktop-portal` (ScreenCast/Screenshot) |
| Diversite des DEs (GNOME, KDE, Hyprland...) | Abstractions dans `common`, backends par DE |
| Permissions root pour certains modules | `pkexec`, capabilities Linux, groupe `input` |
| Overlay transparent | `wlr-layer-shell` (Wayland) / ARGB visual (X11) |
| Clipboard | `wl-clipboard` (Wayland) / `x11rb` selections (X11) |

---

## Config

Toute la config est dans `~/.config/my-power-toys/`.

```
~/.config/my-power-toys/
├── daemon.toml          # Config globale (modules actifs, theme)
├── fancy-zones.toml
├── app-launcher.toml
├── color-picker.toml
├── key-manager.toml
├── ...
└── layouts/             # Layouts FancyZones custom
    ├── default.json
    └── ultrawide.json
```

**Exemple `daemon.toml` :**
```toml
[general]
autostart = true
theme = "system"  # "light", "dark", "system"

[modules]
always-on-top = { enabled = true, hotkey = "Super+T" }
awake = { enabled = true }
color-picker = { enabled = true, hotkey = "Super+Shift+C" }
paste-plain = { enabled = true, hotkey = "Super+Ctrl+V" }
fancy-zones = { enabled = false }
app-launcher = { enabled = true, hotkey = "Super+Space" }
```

---

## Comment Contribuer

1. Chaque module est un crate independant - facile de travailler en parallele
2. Implementer le trait `PowerModule` depuis `common`
3. Ajouter la config TOML par defaut
4. Ecrire des tests unitaires
5. Documenter les permissions requises

---

## Licence

A definir - suggestions : MIT ou GPL-3.0 (GPL recommande pour un outil desktop Linux)
