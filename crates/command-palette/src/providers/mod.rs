pub mod apps;
pub mod calculator;
pub mod files;
pub mod settings;
pub mod shell;
pub mod system;
pub mod web_search;

/// A result from any provider.
#[derive(Debug, Clone)]
pub struct PaletteResult {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: ResultIcon,
    pub action: ResultAction,
    pub score: f64,
    pub provider_tag: &'static str,
}

#[derive(Debug, Clone)]
pub enum ResultIcon {
    Named(String),
    Emoji(char),
    BuiltinApp,
    BuiltinCalc,
    BuiltinWeb,
    BuiltinSystem,
    BuiltinFile,
    BuiltinTerminal,
    BuiltinSettings,
}

#[derive(Debug, Clone)]
pub enum ResultAction {
    LaunchExec(String),
    CopyToClipboard(String),
    OpenUrl(String),
    RunShell(String),
    SystemCommand(SystemCmd),
    OpenSettings(String),
}

#[derive(Debug, Clone)]
pub enum SystemCmd {
    Lock,
    Logout,
    Shutdown,
    Reboot,
    Suspend,
    Hibernate,
}

/// Context passed to providers on each query.
pub struct QueryContext<'a> {
    pub raw_query: &'a str,
    pub stripped_query: &'a str,
    pub max_results: usize,
}

/// Every provider implements this trait.
pub trait Provider: Send {
    fn tag(&self) -> &'static str;
    fn matches(&self, raw_query: &str) -> bool;
    fn strip_prefix<'a>(&self, raw_query: &'a str) -> &'a str;
    fn search(&self, ctx: &QueryContext) -> Vec<PaletteResult>;
    fn initialize(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
