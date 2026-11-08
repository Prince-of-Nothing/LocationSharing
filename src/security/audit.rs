//! Security — Audit logging.
//!
//! Logs security-relevant events for incident response and compliance.
//! Events are written to a local file and can be forwarded to a SIEM.

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// An audit event.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEvent {
    pub timestamp: String,
    pub event_type: String,
    pub actor: Option<String>,
    pub action: String,
    pub resource: Option<String>,
    pub result: String,
    pub details: Option<String>,
    pub source_ip: Option<String>,
}

impl AuditEvent {
    pub fn new(event_type: &str, action: &str, actor: Option<&str>, result: &str) -> Self {
        Self {
            timestamp: now_iso8601(),
            event_type: event_type.to_string(),
            actor: actor.map(|s| s.to_string()),
            action: action.to_string(),
            resource: None,
            result: result.to_string(),
            details: None,
            source_ip: None,
        }
    }

    pub fn with_resource(mut self, resource: &str) -> Self {
        self.resource = Some(resource.to_string());
        self
    }

    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    pub fn with_source_ip(mut self, ip: &str) -> Self {
        self.source_ip = Some(ip.to_string());
        self
    }
}

/// Trait for audit event sinks.
pub trait AuditSink: Send + Sync {
    fn write(&self, event: &AuditEvent) -> std::io::Result<()>;
    fn flush(&self) -> std::io::Result<()>;
}

/// File-based audit sink.
pub struct FileAuditSink {
    writer: Mutex<Option<BufWriter<File>>>,
    #[allow(dead_code)]
    path: PathBuf,
}

impl FileAuditSink {
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        let writer = Some(BufWriter::new(
            OpenOptions::new().create(true).append(true).open(&path)?,
        ));
        Ok(Self {
            writer: Mutex::new(writer),
            path,
        })
    }
}

impl AuditSink for FileAuditSink {
    fn write(&self, event: &AuditEvent) -> std::io::Result<()> {
        let line = serde_json::to_string(event).unwrap_or_default();
        let mut writer = self.writer.lock().unwrap();
        if let Some(w) = writer.as_mut() {
            writeln!(w, "{line}")?;
            w.flush()?;
        }
        Ok(())
    }

    fn flush(&self) -> std::io::Result<()> {
        let mut writer = self.writer.lock().unwrap();
        if let Some(w) = writer.as_mut() {
            w.flush()?;
        }
        Ok(())
    }
}

/// Console audit sink (for development).
pub struct ConsoleAuditSink;

impl AuditSink for ConsoleAuditSink {
    fn write(&self, event: &AuditEvent) -> std::io::Result<()> {
        println!(
            "[AUDIT] {} {} {} {} {}",
            event.timestamp,
            event.event_type,
            event.action,
            event.result,
            event.actor.as_deref().unwrap_or("anonymous")
        );
        Ok(())
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Audit logger that routes events to one or more sinks.
pub struct AuditLogger {
    sinks: Vec<Box<dyn AuditSink>>,
    enabled: bool,
}

impl AuditLogger {
    pub fn new() -> Self {
        Self {
            sinks: Vec::new(),
            enabled: true,
        }
    }

    pub fn with_console() -> Self {
        let mut logger = Self::new();
        logger.add_sink(Box::new(ConsoleAuditSink));
        logger
    }

    pub fn with_file(path: PathBuf) -> std::io::Result<Self> {
        let mut logger = Self::new();
        logger.add_sink(Box::new(FileAuditSink::new(path)?));
        Ok(logger)
    }

    pub fn add_sink(&mut self, sink: Box<dyn AuditSink>) {
        self.sinks.push(sink);
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn log(&self, event: &AuditEvent) {
        if !self.enabled {
            return;
        }
        for sink in &self.sinks {
            if let Err(e) = sink.write(event) {
                eprintln!("[AUDIT] Failed to write audit event: {e}");
            }
        }
    }

    pub fn log_event(&self, event_type: &str, action: &str, actor: Option<&str>, result: &str) {
        let event = AuditEvent::new(event_type, action, actor, result);
        self.log(&event);
    }

    pub fn flush(&self) {
        for sink in &self.sinks {
            let _ = sink.flush();
        }
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Global audit logger instance (set during application initialization).
static AUDIT_LOGGER: std::sync::OnceLock<AuditLogger> = std::sync::OnceLock::new();

/// Initialize the global audit logger.
pub fn init_audit_logger(logger: AuditLogger) {
    let _ = AUDIT_LOGGER.set(logger);
}

/// Get the global audit logger.
pub fn audit_logger() -> Option<&'static AuditLogger> {
    AUDIT_LOGGER.get()
}

/// Log an audit event to the global logger.
pub fn audit_event(event_type: &str, action: &str, actor: Option<&str>, result: &str) {
    if let Some(logger) = audit_logger() {
        logger.log_event(event_type, action, actor, result);
    }
}

fn now_iso8601() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| {
            let secs = d.as_secs();
            let nanos = d.subsec_nanos();
            let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, nanos)
                .unwrap_or_else(chrono::Utc::now);
            dt.to_rfc3339()
        })
        .unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_event_creation() {
        let event = AuditEvent::new("auth", "login", Some("alice"), "success")
            .with_resource("/api/v1/auth/login")
            .with_details("user authenticated")
            .with_source_ip("192.168.1.1");

        assert_eq!(event.event_type, "auth");
        assert_eq!(event.action, "login");
        assert_eq!(event.actor, Some("alice".to_string()));
        assert_eq!(event.result, "success");
        assert_eq!(event.resource, Some("/api/v1/auth/login".to_string()));
        assert_eq!(event.details, Some("user authenticated".to_string()));
        assert_eq!(event.source_ip, Some("192.168.1.1".to_string()));
        assert!(!event.timestamp.is_empty());
    }

    #[test]
    fn console_sink_writes() {
        let sink = ConsoleAuditSink;
        let event = AuditEvent::new("auth", "login", None, "success");
        assert!(sink.write(&event).is_ok());
        assert!(sink.flush().is_ok());
    }

    #[test]
    fn file_sink_writes() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let path = temp.path().to_path_buf();
        let sink = FileAuditSink::new(path.clone()).unwrap();

        let event = AuditEvent::new("auth", "login", Some("alice"), "success");
        assert!(sink.write(&event).is_ok());

        // Verify file was written
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("alice"));
        assert!(contents.contains("login"));
    }

    #[test]
    fn audit_logger_with_console() {
        let logger = AuditLogger::with_console();
        logger.log_event("auth", "login", Some("bob"), "success");
        logger.flush();
    }

    #[test]
    fn global_audit_logger() {
        let logger = AuditLogger::with_console();
        init_audit_logger(logger);
        audit_event("auth", "logout", None, "success");
    }
}
