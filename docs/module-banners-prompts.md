# MyPowerToys - Prompts bannieres style PowerToys

Ce document est une V2 orientee "look PowerToys" basee sur des references visuelles:
- banniere large type hero en haut de page,
- scene logicielle realiste (pas trop abstraite),
- focus central propre pour le logo du module,
- rendu sobre, premium, lisible.

## Recette visuelle "PowerToys-like"

1. Ratio large: `3:1` (ex: `1920x640`).
2. Scene semi-realiste: desktop/UI elements, surfaces, overlays, reflets doux.
3. Profondeur legere: flou de fond subtil, sujet principal net.
4. Palette neutre + accent couleur module.
5. Zone centrale propre pour le logo.
6. Aucun texte rendu dans l'image.

## Contexte global (a fournir une fois)

```text
Projet: MyPowerToys, application desktop Linux inspiree de Microsoft PowerToys.
Objectif: une banniere hero par module dans la page detail module.
Direction artistique: software product banner, moderne, propre, semi-realiste, premium.
Contraintes: image sans texte, sans watermark, sans elements parasites.
```

## Prompt maitre recommande (fond uniquement, logo ajoute ensuite)

Cette version est la plus fiable si tu veux un logo parfaitement propre au centre.

```text
PowerToys-style hero banner for a desktop utility module, wide 3:1 composition (1920x640), clean software marketing artwork, semi-realistic digital matte scene with subtle UI context, soft depth of field, gentle cinematic lighting, neutral cool-gray base palette with accent color {ACCENT_HEX}, clear central focal zone reserved for a module logo overlay, coherent geometric perspective, polished and minimal, {MODULE_SCENE}, no text, no letters, no watermark, no people, no clutter.
```

## Prompt maitre alternatif (logo genere dans l'image)

```text
PowerToys-style hero banner for a desktop utility module, wide 3:1 composition (1920x640), clean software marketing artwork, semi-realistic digital matte scene with subtle UI context, soft depth of field, gentle cinematic lighting, neutral cool-gray base palette with accent color {ACCENT_HEX}, centered floating icon tile representing the module, rounded square badge, soft shadow, subtle glow, polished and minimal, {MODULE_SCENE}, no text, no letters, no watermark, no people, no clutter.
```

## Negative prompt (si supporte)

```text
text, letters, words, watermark, logo artifacts, over-detailed busy scene, extreme saturation, low contrast, blur on main subject, noisy image, human faces, photoreal people, UI screenshot
```

## Liste des bannieres a generer

| Module ID | Nom module | Fichier image | Accent | MODULE_SCENE |
| --- | --- | --- | --- | --- |
| `always-on-top` | Always on Top | `always-on-top-banner.png` | `#89B4FA` | `layered desktop windows, one active window floating above all others, clear stacking depth, pin-like visual cue` |
| `awake` | Awake | `awake-banner.png` | `#CBA6F7` | `night-to-day transition mood, subtle moon haze fading into active glow, sleep state being prevented` |
| `paste-plain` | Paste as Plain Text | `paste-plain-banner.png` | `#A6E3A1` | `rich formatted clipboard content transforming into clean plain text blocks, formatting marks dissolving` |
| `color-picker` | Color Picker | `color-picker-banner.png` | `#94E2D5` | `screen color sampling scene, eyedropper focus, soft color swatch card, clean UI-like composition` |
| `hosts-editor` | Hosts Editor | `hosts-editor-banner.png` | `#FAB387` | `network nodes and host routes with editable file panel feeling, toggled entries, controlled admin utility vibe` |
| `bulk-rename` | Bulk Rename | `bulk-rename-banner.png` | `#F5C2E7` | `multiple file cards in sequence, rename transformation flow, before/after organization` |
| `image-resizer` | Image Resizer | `image-resizer-banner.png` | `#74C7EC` | `image thumbnails with resize handles and scaling guides, precise dimensions feel, batch workflow` |
| `key-manager` | Key Manager | `key-manager-banner.png` | `#B4BEFE` | `keyboard keycaps linked by remap arrows, shortcut remapping paths, structured input control` |
| `mouse-utils` | Mouse Utilities | `mouse-utils-banner.png` | `#F9E2AF` | `cursor spotlight ring, click ripple markers, crosshair/locator hints on a clean desktop surface` |
| `screen-ruler` | Screen Ruler | `screen-ruler-banner.png` | `#F2CDCD` | `pixel ruler overlays, measurement ticks and brackets on UI surface, precise alignment tool mood` |
| `text-extractor` | Text Extractor | `text-extractor-banner.png` | `#89DCEB` | `OCR capture frame over screen region, extracted text blocks appearing as clean neutral glyph bars` |
| `shortcut-guide` | Shortcut Guide | `shortcut-guide-banner.png` | `#F9E2AF` | `keyboard centered context with radial shortcut hint overlays, guidance layer, no readable labels` |
| `app-launcher` | App Launcher | `app-launcher-banner.png` | `#F5E0DC` | `search-first launcher scene, app tiles emerging from a central query surface, quick access mood` |
| `fancy-zones` | FancyZones | `fancy-zones-banner.png` | `#89B4FA` | `window tiling grid layout across desktop, snap trajectories into custom zones, structured composition` |
| `peek` | Peek | `peek-banner.png` | `#EBA0AC` | `quick preview pane over files, eye-focus metaphor with clean preview cards for media/text/pdf` |
| `light-switch` | Light Switch | `light-switch-banner.png` | `#F9E2AF` | `day-to-night theme transition scene, split desktop surface with light mode fading into dark mode, toggle switch metaphor with warm-to-cool color gradient, sunrise and sunset ambient cues` |

## Exemple complet (Color Picker)

```text
PowerToys-style hero banner for a desktop utility module, wide 3:1 composition (1920x640), clean software marketing artwork, semi-realistic digital matte scene with subtle UI context, soft depth of field, gentle cinematic lighting, neutral cool-gray base palette with accent color #94E2D5, clear central focal zone reserved for a module logo overlay, coherent geometric perspective, polished and minimal, screen color sampling scene, eyedropper focus, soft color swatch card, clean UI-like composition, no text, no letters, no watermark, no people, no clutter.
```

## Checklist qualite avant validation

1. Le centre est exploitable pour le logo.
2. Le module est reconnaissable sans texte.
3. Le rendu reste sobre (pas surcharge).
4. Le style est coherent avec les autres bannieres.
5. Le contraste est bon sur ecran clair et sombre.
