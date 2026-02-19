use super::{PaletteResult, Provider, QueryContext, ResultAction, ResultIcon, SystemCmd};

struct SystemEntry {
    keywords: &'static [&'static str],
    title: &'static str,
    subtitle: &'static str,
    cmd: SystemCmd,
}

const SYSTEM_COMMANDS: &[SystemEntry] = &[
    SystemEntry {
        keywords: &["lock", "lock screen"],
        title: "Lock Screen",
        subtitle: "loginctl lock-session",
        cmd: SystemCmd::Lock,
    },
    SystemEntry {
        keywords: &["logout", "log out", "sign out"],
        title: "Log Out",
        subtitle: "End current session",
        cmd: SystemCmd::Logout,
    },
    SystemEntry {
        keywords: &["shutdown", "power off", "poweroff"],
        title: "Shut Down",
        subtitle: "systemctl poweroff",
        cmd: SystemCmd::Shutdown,
    },
    SystemEntry {
        keywords: &["reboot", "restart"],
        title: "Reboot",
        subtitle: "systemctl reboot",
        cmd: SystemCmd::Reboot,
    },
    SystemEntry {
        keywords: &["suspend", "sleep"],
        title: "Suspend",
        subtitle: "systemctl suspend",
        cmd: SystemCmd::Suspend,
    },
    SystemEntry {
        keywords: &["hibernate"],
        title: "Hibernate",
        subtitle: "systemctl hibernate",
        cmd: SystemCmd::Hibernate,
    },
];

pub struct SystemProvider;

impl SystemProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for SystemProvider {
    fn tag(&self) -> &'static str {
        "system"
    }

    fn matches(&self, raw_query: &str) -> bool {
        // System provider matches when there's no prefix (same as apps)
        !raw_query.starts_with('>')
            && !raw_query.starts_with("??")
            && !raw_query.starts_with('=')
            && !raw_query.starts_with('$')
            && !raw_query.starts_with('/')
            && !raw_query.starts_with("file ")
    }

    fn strip_prefix<'a>(&self, raw_query: &'a str) -> &'a str {
        raw_query
    }

    fn search(&self, ctx: &QueryContext) -> Vec<PaletteResult> {
        let query = ctx.stripped_query.trim().to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }

        SYSTEM_COMMANDS
            .iter()
            .filter_map(|entry| {
                let best_score = entry
                    .keywords
                    .iter()
                    .filter_map(|kw| {
                        if *kw == query {
                            Some(70.0)
                        } else if kw.starts_with(&query) {
                            Some(55.0)
                        } else if kw.contains(&query) {
                            Some(40.0)
                        } else {
                            None
                        }
                    })
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                best_score.map(|score| PaletteResult {
                    id: format!("system:{}", entry.title),
                    title: entry.title.to_string(),
                    subtitle: Some(entry.subtitle.to_string()),
                    icon: ResultIcon::BuiltinSystem,
                    action: ResultAction::SystemCommand(entry.cmd.clone()),
                    score,
                    provider_tag: "system",
                })
            })
            .collect()
    }
}
