use crate::modules::ModuleRegistry;
use anyhow::{Context, Result, anyhow, bail};
use mpt_common::hotkey::{Hotkey, Modifier};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{debug, info, warn};
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{ConnectionExt, GrabMode, ModMask, Window};
use x11rb::rust_connection::RustConnection;

/// Minimum interval between two hotkey activations for the same module (ms).
const HOTKEY_COOLDOWN_MS: u128 = 500;

const MOD_SHIFT: u16 = 1 << 0;
const MOD_LOCK: u16 = 1 << 1;
const MOD_CONTROL: u16 = 1 << 2;
const MOD_ALT: u16 = 1 << 3;
const MOD_NUMLOCK: u16 = 1 << 4;
const MOD_SUPER: u16 = 1 << 6;

#[derive(Debug, Clone)]
struct RegisteredHotkey {
    module_id: String,
    display_hotkey: String,
}

pub fn spawn_x11_hotkey_listener(
    bindings: Vec<(String, String)>,
    registry: Arc<Mutex<ModuleRegistry>>,
) {
    std::thread::spawn(move || {
        if let Err(err) = run_x11_hotkey_listener(bindings, registry) {
            warn!("Global X11 hotkey listener stopped: {err}");
        }
    });
}

fn run_x11_hotkey_listener(
    bindings: Vec<(String, String)>,
    registry: Arc<Mutex<ModuleRegistry>>,
) -> Result<()> {
    if bindings.is_empty() {
        info!("No hotkey bindings configured for global listener");
        return Ok(());
    }

    let (conn, screen_num) =
        RustConnection::connect(None).context("failed to connect to X11 for global hotkeys")?;
    let root = conn.setup().roots[screen_num].root;

    let mut triggers: HashMap<(u8, u16), RegisteredHotkey> = HashMap::new();
    let mut registered_count = 0usize;

    for (module_id, raw_hotkey) in bindings {
        let parsed = match parse_hotkey_notation(&raw_hotkey) {
            Ok(hk) => hk,
            Err(err) => {
                warn!("Skipping hotkey '{raw_hotkey}' for module '{module_id}': {err}");
                continue;
            }
        };

        let keysyms = hotkey_key_to_keysyms(&parsed.key);
        if keysyms.is_empty() {
            warn!(
                "Skipping hotkey '{raw_hotkey}' for module '{module_id}': unsupported key '{}'",
                parsed.key
            );
            continue;
        }

        let keycodes = keycodes_for_keysyms(&conn, &keysyms)?;
        if keycodes.is_empty() {
            warn!(
                "Skipping hotkey '{raw_hotkey}' for module '{module_id}': key not found in keyboard map"
            );
            continue;
        }

        let modifiers = modifiers_to_x11(&parsed.modifiers);
        for keycode in keycodes {
            if let Err(err) = grab_with_lock_variants(&conn, root, keycode, modifiers) {
                warn!(
                    "Failed to grab '{raw_hotkey}' (keycode {keycode}) for module '{module_id}': {err}"
                );
                continue;
            }

            let trigger_key = (keycode, normalize_mod_state(modifiers));
            let new_hotkey = RegisteredHotkey {
                module_id: module_id.clone(),
                display_hotkey: raw_hotkey.clone(),
            };

            if let Some(existing) = triggers.insert(trigger_key, new_hotkey) {
                warn!(
                    "Hotkey conflict on keycode {keycode}: '{}' ({}) replaced '{}' ({})",
                    raw_hotkey, module_id, existing.display_hotkey, existing.module_id
                );
            } else {
                registered_count += 1;
            }
        }
    }

    conn.flush().context("failed to flush X11 hotkey grabs")?;
    info!("Global X11 hotkeys active: {registered_count} binding(s)");

    // Track last activation time per module to debounce key-repeat events
    let mut last_activation: HashMap<String, Instant> = HashMap::new();

    loop {
        let event = conn
            .wait_for_event()
            .context("X11 hotkey event loop error")?;
        if let Event::KeyPress(event) = event {
            let keycode = event.detail;
            let state = normalize_mod_state(u16::from(event.state));
            if let Some(binding) = triggers.get(&(keycode, state)).cloned() {
                // Debounce: skip if the same module was activated too recently
                let now = Instant::now();
                if let Some(last) = last_activation.get(&binding.module_id)
                    && now.duration_since(*last).as_millis() < HOTKEY_COOLDOWN_MS
                {
                    debug!(
                        "Hotkey '{}' for '{}' suppressed (cooldown)",
                        binding.display_hotkey, binding.module_id
                    );
                    continue;
                }
                last_activation.insert(binding.module_id.clone(), now);

                debug!(
                    "Global hotkey pressed: '{}' -> {}",
                    binding.display_hotkey, binding.module_id
                );

                let mut reg = registry.lock().unwrap();
                if let Err(err) = reg.trigger_hotkey(&binding.module_id) {
                    warn!(
                        "Hotkey '{}' for module '{}' failed: {err}",
                        binding.display_hotkey, binding.module_id
                    );
                }
            }
        }
    }
}

fn parse_hotkey_notation(raw: &str) -> Result<Hotkey> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("empty hotkey");
    }

    let lower = trimmed.to_lowercase();
    if lower.starts_with("hold ") {
        let key = trimmed[5..].trim();
        if key.is_empty() {
            bail!("invalid hold hotkey");
        }
        return Ok(Hotkey::new(Vec::new(), normalize_key_token(key)));
    }

    let mut hotkey =
        Hotkey::parse(trimmed).with_context(|| format!("invalid hotkey '{trimmed}'"))?;
    hotkey.key = normalize_key_token(&hotkey.key);

    // Best-effort handling for strings like "Ctrl+Ctrl": treat as a single key press.
    if hotkey.modifiers.len() == 1 && modifier_matches_key(hotkey.modifiers[0], &hotkey.key) {
        hotkey.modifiers.clear();
    }

    Ok(hotkey)
}

fn normalize_key_token(raw: &str) -> String {
    match raw.trim().to_lowercase().as_str() {
        "control" | "ctrl" => "Ctrl".to_string(),
        "super" | "meta" | "win" => "Super".to_string(),
        "alt" => "Alt".to_string(),
        "shift" => "Shift".to_string(),
        "space" => "Space".to_string(),
        other => {
            if other.len() == 1 {
                other.to_uppercase()
            } else {
                raw.trim().to_string()
            }
        }
    }
}

fn modifier_matches_key(modifier: Modifier, key: &str) -> bool {
    match modifier {
        Modifier::Ctrl => key.eq_ignore_ascii_case("ctrl"),
        Modifier::Alt => key.eq_ignore_ascii_case("alt"),
        Modifier::Shift => key.eq_ignore_ascii_case("shift"),
        Modifier::Super => key.eq_ignore_ascii_case("super"),
    }
}

fn modifiers_to_x11(modifiers: &[Modifier]) -> u16 {
    let mut mask = 0u16;
    for modifier in modifiers {
        mask |= match modifier {
            Modifier::Shift => MOD_SHIFT,
            Modifier::Ctrl => MOD_CONTROL,
            Modifier::Alt => MOD_ALT,
            Modifier::Super => MOD_SUPER,
        };
    }
    mask
}

fn normalize_mod_state(state: u16) -> u16 {
    let keyboard_mods = state & 0x00ff;
    keyboard_mods & !(MOD_LOCK | MOD_NUMLOCK)
}

fn grab_with_lock_variants(
    conn: &RustConnection,
    root: Window,
    keycode: u8,
    modifiers: u16,
) -> Result<()> {
    let variants = [
        modifiers,
        modifiers | MOD_LOCK,
        modifiers | MOD_NUMLOCK,
        modifiers | MOD_LOCK | MOD_NUMLOCK,
    ];

    for state in variants {
        conn.grab_key(
            false,
            root,
            ModMask::from(state),
            keycode,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        )
        .context("grab_key request failed")?
        .check()
        .map_err(|err| anyhow!(err))
        .context("grab_key rejected by X server")?;
    }

    Ok(())
}

fn keycodes_for_keysyms(conn: &RustConnection, target_keysyms: &[u32]) -> Result<Vec<u8>> {
    let setup = conn.setup();
    let min = setup.min_keycode;
    let max = setup.max_keycode;
    let count = max.saturating_sub(min).saturating_add(1);

    let reply = conn
        .get_keyboard_mapping(min, count)
        .context("get_keyboard_mapping request failed")?
        .reply()
        .context("get_keyboard_mapping reply failed")?;

    if reply.keysyms_per_keycode == 0 {
        return Ok(Vec::new());
    }

    let per_keycode = reply.keysyms_per_keycode as usize;
    let mut keycodes = Vec::new();
    for (offset, keysyms) in reply.keysyms.chunks(per_keycode).enumerate() {
        if keysyms.iter().any(|keysym| target_keysyms.contains(keysym)) {
            keycodes.push(min.saturating_add(offset as u8));
        }
    }
    keycodes.sort_unstable();
    keycodes.dedup();
    Ok(keycodes)
}

fn hotkey_key_to_keysyms(key: &str) -> Vec<u32> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return vec![];
    }

    if trimmed.len() == 1 {
        let ch = trimmed.chars().next().unwrap();
        if ch.is_ascii_alphabetic() {
            return vec![
                ch.to_ascii_uppercase() as u32,
                ch.to_ascii_lowercase() as u32,
            ];
        }
        return vec![ch as u32];
    }

    match trimmed.to_lowercase().as_str() {
        "space" => vec![xkeysym::key::space, xkeysym::key::KP_Space],
        "tab" => vec![xkeysym::key::Tab, xkeysym::key::KP_Tab],
        "enter" | "return" => vec![xkeysym::key::Return, xkeysym::key::KP_Enter],
        "esc" | "escape" => vec![xkeysym::key::Escape],
        "backspace" => vec![xkeysym::key::BackSpace],
        "delete" | "del" => vec![xkeysym::key::Delete, xkeysym::key::KP_Delete],
        "ctrl" | "control" => vec![xkeysym::key::Control_L, xkeysym::key::Control_R],
        "alt" => vec![xkeysym::key::Alt_L, xkeysym::key::Alt_R],
        "shift" => vec![xkeysym::key::Shift_L, xkeysym::key::Shift_R],
        "super" | "meta" | "win" => vec![xkeysym::key::Super_L, xkeysym::key::Super_R],
        "left" => vec![xkeysym::key::Left],
        "right" => vec![xkeysym::key::Right],
        "up" => vec![xkeysym::key::Up],
        "down" => vec![xkeysym::key::Down],
        "f1" => vec![xkeysym::key::F1],
        "f2" => vec![xkeysym::key::F2],
        "f3" => vec![xkeysym::key::F3],
        "f4" => vec![xkeysym::key::F4],
        "f5" => vec![xkeysym::key::F5],
        "f6" => vec![xkeysym::key::F6],
        "f7" => vec![xkeysym::key::F7],
        "f8" => vec![xkeysym::key::F8],
        "f9" => vec![xkeysym::key::F9],
        "f10" => vec![xkeysym::key::F10],
        "f11" => vec![xkeysym::key::F11],
        "f12" => vec![xkeysym::key::F12],
        _ => vec![],
    }
}
