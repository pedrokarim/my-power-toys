//! Internationalisation strings for MyPowerToys Settings UI.

mod cn;
mod en;
mod es;
mod fr;
mod jp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Fr,
    En,
    Es,
    Jp,
    Cn,
}

pub struct Tr {
    // Sidebar & general
    pub settings: &'static str,
    pub dashboard: &'static str,
    pub modules_label: &'static str,
    pub about: &'static str,
    pub preferences: &'static str,
    pub tests: &'static str,

    // Dashboard
    pub daemon_connected: &'static str,
    pub daemon_not_connected: &'static str,
    pub total: &'static str,
    pub active: &'static str,
    pub inactive: &'static str,
    pub all_modules: &'static str,

    // Preferences
    pub appearance: &'static str,
    pub theme_desc: &'static str,
    pub light: &'static str,
    pub dark: &'static str,
    pub auto_theme: &'static str,
    pub language: &'static str,
    pub lang_desc: &'static str,
    pub text_size: &'static str,
    pub text_size_desc: &'static str,
    pub small: &'static str,
    pub medium: &'static str,
    pub large: &'static str,

    // Accessibility
    pub accessibility: &'static str,
    pub high_contrast: &'static str,
    pub high_contrast_desc: &'static str,
    pub bold_text: &'static str,
    pub bold_text_desc: &'static str,
    pub compact_layout: &'static str,
    pub compact_layout_desc: &'static str,
    pub reduced_motion: &'static str,
    pub reduced_motion_desc: &'static str,

    // Module detail
    pub status: &'static str,
    pub running: &'static str,
    pub stopped: &'static str,
    pub enabled: &'static str,
    pub hotkey: &'static str,
    pub module_settings: &'static str,
    pub module_settings_placeholder: &'static str,
    pub module_not_found: &'static str,

    // Tests
    pub tests_title: &'static str,
    pub tests_desc: &'static str,
    pub tests_keys_title: &'static str,
    pub tests_keys_desc: &'static str,
    pub tests_active_keys: &'static str,
    pub tests_start_btn: &'static str,
    pub tests_stop_btn: &'static str,
    pub tests_no_keys: &'static str,
    pub test_action: &'static str,
    pub test_result: &'static str,
    pub tests_no_hotkey: &'static str,
    pub daemon_required: &'static str,
    pub tests_wayland_hint: &'static str,
    pub no_shortcuts: &'static str,
    pub deps_help: &'static str,
    pub deps_hide: &'static str,
    pub deps_title: &'static str,
    pub deps_for_system: &'static str,
    pub deps_command: &'static str,
    pub deps_notes: &'static str,
    pub deps_copy: &'static str,
    pub deps_continue: &'static str,
    pub toast_success_title: &'static str,
    pub toast_error_title: &'static str,
    pub toast_command_copied: &'static str,
    pub toast_copy_failed: &'static str,
    pub toast_update_success: &'static str,
    pub toast_update_already: &'static str,
    pub toast_update_failed: &'static str,

    // About
    pub about_title: &'static str,
    pub about_desc: &'static str,
    pub about_detail: &'static str,
    pub update_section: &'static str,
    pub update_current_version: &'static str,
    pub update_latest_version: &'static str,
    pub update_status: &'static str,
    pub update_not_checked: &'static str,
    pub update_checking: &'static str,
    pub update_up_to_date: &'static str,
    pub update_available: &'static str,
    pub update_updating: &'static str,
    pub update_error: &'static str,
    pub update_check: &'static str,
    pub update_install: &'static str,
    pub update_dialog_title: &'static str,
    pub update_dialog_body: &'static str,
    pub update_dialog_cancel: &'static str,
    pub update_dialog_confirm: &'static str,
    pub tech_stack: &'static str,
    pub author: &'static str,
    pub license: &'static str,
    pub repo: &'static str,
    pub prog_lang: &'static str,
}

pub fn get(lang: Language) -> &'static Tr {
    match lang {
        Language::En => &en::EN,
        Language::Fr => &fr::FR,
        Language::Es => &es::ES,
        Language::Jp => &jp::JP,
        Language::Cn => &cn::CN,
    }
}
