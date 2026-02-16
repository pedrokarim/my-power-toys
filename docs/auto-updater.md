# Auto-Updater

MyPowerToys inclut un systeme de mise a jour automatique qui verifie les nouvelles releases sur GitHub et remplace les binaires en place.

## Fonctionnement

L'updater interroge l'API GitHub (`pedrokarim/my-power-toys/releases`) pour comparer la version installee avec la derniere release disponible. Si une nouvelle version existe, il telecharge l'asset correspondant et remplace le binaire de maniere atomique.

**Format d'asset attendu :** `my-power-toys-<version>-<target>.tar.gz`

Par exemple : `my-power-toys-0.2.0-x86_64-unknown-linux-gnu.tar.gz`

## Commandes CLI

### Verifier les mises a jour

```bash
mpt-ctl check-update
```

Interroge GitHub et affiche si une nouvelle version est disponible, avec les notes de release.

**Exemples de sortie :**

```
Checking for updates...
Update available: 0.1.0 -> 0.2.0

Release notes:
- Ajout du color picker
- Correction de bugs dans always-on-top

Run `mpt-ctl update` to install.
```

```
Checking for updates...
Already up to date (v0.1.0).
```

### Installer la mise a jour

```bash
mpt-ctl update
```

Telecharge la derniere version et remplace le binaire `mpt-ctl`. Apres la mise a jour, redemarrer le daemon :

```bash
systemctl --user restart mpt-daemon
```

## Interface D-Bus

Le daemon expose deux methodes supplementaires sur `org.mypowertoys.Daemon` :

| Methode | Retour | Description |
|---|---|---|
| `GetVersion()` | `"0.1.0"` | Version courante du daemon |
| `CheckForUpdates()` | `"up-to-date"` ou `"available:0.2.0"` | Verifie les mises a jour |

### Exemple avec `busctl`

```bash
busctl --user call org.mypowertoys.Daemon /org/mypowertoys/Daemon org.mypowertoys.Daemon GetVersion
busctl --user call org.mypowertoys.Daemon /org/mypowertoys/Daemon org.mypowertoys.Daemon CheckForUpdates
```

## Architecture

Le code est organise ainsi :

- `crates/common/src/updater.rs` — logique partagee (check + update)
- `crates/cli/src/main.rs` — commandes `check-update` et `update`
- `crates/daemon/src/dbus.rs` — methodes D-Bus `GetVersion` et `CheckForUpdates`

La crate `self_update` est utilisee comme backend. Elle gere :
- Les requetes a l'API GitHub Releases
- La comparaison de versions (semver)
- Le telechargement des assets `.tar.gz`
- Le remplacement atomique du binaire via `self-replace`

## Publier une release

Pour que l'auto-updater fonctionne, chaque release GitHub doit inclure un asset `.tar.gz` contenant les binaires compiles. Exemple de workflow :

```bash
cargo build --release
tar czf my-power-toys-0.2.0-x86_64-unknown-linux-gnu.tar.gz \
  -C target/release mpt-daemon mpt-ctl mpt-settings
gh release create v0.2.0 my-power-toys-0.2.0-x86_64-unknown-linux-gnu.tar.gz
```
