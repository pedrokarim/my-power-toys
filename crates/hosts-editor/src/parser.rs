use anyhow::{Context, Result};
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct HostEntry {
    pub ip: String,
    pub hostnames: Vec<String>,
    pub comment: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum HostsLine {
    Entry(HostEntry),
    Comment(String),
    Blank,
}

#[derive(Debug, Clone)]
pub struct HostsFile {
    pub lines: Vec<HostsLine>,
}

impl HostsFile {
    /// Parse /etc/hosts (or any hosts file).
    pub fn parse(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        Ok(Self::parse_str(&content))
    }

    pub fn parse_str(content: &str) -> Self {
        let lines = content.lines().map(parse_line).collect();
        Self { lines }
    }

    /// Get all host entries (both enabled and disabled).
    pub fn entries(&self) -> Vec<&HostEntry> {
        self.lines
            .iter()
            .filter_map(|l| match l {
                HostsLine::Entry(e) => Some(e),
                _ => None,
            })
            .collect()
    }

    /// Toggle an entry on/off by index (among entries only).
    pub fn toggle_entry(&mut self, entry_index: usize) -> Result<()> {
        let mut count = 0;
        for line in &mut self.lines {
            if let HostsLine::Entry(entry) = line {
                if count == entry_index {
                    entry.enabled = !entry.enabled;
                    return Ok(());
                }
                count += 1;
            }
        }
        anyhow::bail!("entry index {entry_index} out of range (have {count} entries)");
    }

    /// Add a new entry.
    pub fn add_entry(&mut self, ip: String, hostnames: Vec<String>, comment: Option<String>) {
        self.lines.push(HostsLine::Entry(HostEntry {
            ip,
            hostnames,
            comment,
            enabled: true,
        }));
    }

    /// Remove an entry by index.
    pub fn remove_entry(&mut self, entry_index: usize) -> Result<()> {
        let mut count = 0;
        let mut line_index = None;
        for (i, line) in self.lines.iter().enumerate() {
            if matches!(line, HostsLine::Entry(_)) {
                if count == entry_index {
                    line_index = Some(i);
                    break;
                }
                count += 1;
            }
        }
        match line_index {
            Some(i) => {
                self.lines.remove(i);
                Ok(())
            }
            None => anyhow::bail!("entry index {entry_index} out of range"),
        }
    }

    /// Serialize back to hosts file format.
    pub fn serialize(&self) -> String {
        self.lines
            .iter()
            .map(|l| match l {
                HostsLine::Entry(e) => format_entry(e),
                HostsLine::Comment(c) => c.clone(),
                HostsLine::Blank => String::new(),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Save to a file. For /etc/hosts, the caller should handle privilege elevation.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = self.serialize();
        fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))?;
        Ok(())
    }
}

fn parse_line(line: &str) -> HostsLine {
    let trimmed = line.trim();

    if trimmed.is_empty() {
        return HostsLine::Blank;
    }

    // Check if this is a commented-out entry (disabled) vs a pure comment
    if let Some(rest) = trimmed.strip_prefix('#') {
        let rest = rest.trim();
        // Try to parse as a disabled entry (has an IP-like start)
        if looks_like_host_entry(rest)
            && let Some(entry) = try_parse_entry(rest)
        {
            return HostsLine::Entry(HostEntry {
                enabled: false,
                ..entry
            });
        }
        return HostsLine::Comment(line.to_string());
    }

    if let Some(entry) = try_parse_entry(trimmed) {
        HostsLine::Entry(entry)
    } else {
        HostsLine::Comment(line.to_string())
    }
}

fn looks_like_host_entry(s: &str) -> bool {
    let first = s.split_whitespace().next().unwrap_or("");
    first.contains('.') || first.contains(':') // IPv4 or IPv6
}

fn try_parse_entry(line: &str) -> Option<HostEntry> {
    // Split off inline comment
    let (main, comment) = match line.find('#') {
        Some(pos) => (&line[..pos], Some(line[pos + 1..].trim().to_string())),
        None => (line, None),
    };

    let parts: Vec<&str> = main.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    Some(HostEntry {
        ip: parts[0].to_string(),
        hostnames: parts[1..].iter().map(|s| s.to_string()).collect(),
        comment,
        enabled: true,
    })
}

fn format_entry(entry: &HostEntry) -> String {
    let prefix = if entry.enabled { "" } else { "# " };
    let hosts = entry.hostnames.join(" ");
    match &entry.comment {
        Some(c) => format!("{prefix}{}\t{hosts} # {c}", entry.ip),
        None => format!("{prefix}{}\t{hosts}", entry.ip),
    }
}

impl fmt::Display for HostsFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.serialize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
# /etc/hosts
127.0.0.1\tlocalhost
127.0.1.1\tmy-machine
# ::1\tlocalhost ip6-localhost
192.168.1.100\tdevserver # dev box

# The following lines are for IPv6
::1\tip6-localhost ip6-loopback
";

    #[test]
    fn parse_hosts_file() {
        let hosts = HostsFile::parse_str(SAMPLE);
        let entries = hosts.entries();
        assert_eq!(entries.len(), 5);
        assert!(entries[0].enabled); // 127.0.0.1 localhost
        assert!(entries[1].enabled); // 127.0.1.1 my-machine
        assert!(!entries[2].enabled); // # ::1 localhost (disabled)
        assert!(entries[3].enabled); // 192.168.1.100 devserver
        assert!(entries[4].enabled); // ::1 ip6-localhost
    }

    #[test]
    fn toggle_entry() {
        let mut hosts = HostsFile::parse_str(SAMPLE);
        assert!(hosts.entries()[0].enabled);
        hosts.toggle_entry(0).unwrap();
        assert!(!hosts.entries()[0].enabled);
        hosts.toggle_entry(0).unwrap();
        assert!(hosts.entries()[0].enabled);
    }

    #[test]
    fn add_and_remove() {
        let mut hosts = HostsFile::parse_str("");
        hosts.add_entry("10.0.0.1".to_string(), vec!["test.local".to_string()], None);
        assert_eq!(hosts.entries().len(), 1);
        hosts.remove_entry(0).unwrap();
        assert_eq!(hosts.entries().len(), 0);
    }

    #[test]
    fn roundtrip_preserves_comments() {
        let hosts = HostsFile::parse_str(SAMPLE);
        let output = hosts.to_string();
        assert!(output.contains("# /etc/hosts"));
        assert!(output.contains("# The following lines are for IPv6"));
    }

    #[test]
    fn inline_comment_preserved() {
        let hosts = HostsFile::parse_str("192.168.1.100\tdevserver # dev box");
        let entry = &hosts.entries()[0];
        assert_eq!(entry.comment.as_deref(), Some("dev box"));
    }
}
