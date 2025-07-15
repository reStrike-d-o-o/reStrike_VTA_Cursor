// Comprehensive Logging System for reStrike VTA Backend
// This system provides centralized logging with file output and automatic cleanup

use chrono::{DateTime, Utc};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    DEBUG = 0,
    INFO = 1,
    WARN = 2,
    ERROR = 3,
    CRITICAL = 4,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::DEBUG => "DEBUG",
            LogLevel::INFO => "INFO",
            LogLevel::WARN => "WARN",
            LogLevel::ERROR => "ERROR",
            LogLevel::CRITICAL => "CRITICAL",
        }
    }

    fn emoji(&self) -> &'static str {
        match self {
            LogLevel::DEBUG => "🔍",
            LogLevel::INFO => "ℹ️",
            LogLevel::WARN => "⚠️",
            LogLevel::ERROR => "❌",
            LogLevel::CRITICAL => "🚨",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub component: Option<String>,
    pub data: Option<String>,
}

pub struct Logger {
    log_file: Option<Arc<Mutex<File>>>,
    log_level: LogLevel,
    log_buffer: Arc<Mutex<Vec<LogEntry>>>,
    max_buffer_size: usize,
}

impl Logger {
    pub fn new() -> Self {
        let mut logger = Self {
            log_file: None,
            log_level: LogLevel::INFO,
            log_buffer: Arc::new(Mutex::new(Vec::new())),
            max_buffer_size: 1000,
        };

        logger.cleanup_log_file();
        logger.setup_log_file();
        logger
    }

    fn cleanup_log_file(&self) {
        let log_path = "logs/restrike-vta.log";
        if Path::new(log_path).exists() {
            // In a production system, you might want to rotate logs instead of deleting
            if let Err(e) = std::fs::remove_file(log_path) {
                eprintln!("Failed to cleanup log file: {}", e);
            }
        }

        // Ensure logs directory exists
        if let Err(e) = std::fs::create_dir_all("logs") {
            eprintln!("Failed to create logs directory: {}", e);
        }
    }

    fn setup_log_file(&mut self) {
        let log_path = "logs/restrike-vta.log";
        match OpenOptions::new().create(true).append(true).open(log_path) {
            Ok(file) => {
                self.log_file = Some(Arc::new(Mutex::new(file)));
                self.info("Log file initialized", None, Some("Logger"));
            }
            Err(e) => {
                eprintln!("Failed to open log file: {}", e);
            }
        }
    }

    pub fn set_log_level(&mut self, level: LogLevel) {
        self.log_level = level;
        self.info(
            &format!("Log level set to: {}", level.as_str()),
            None,
            Some("Logger"),
        );
    }

    pub fn debug(&self, message: &str, data: Option<&str>, component: Option<&str>) {
        self.log(LogLevel::DEBUG, message, data, component);
    }

    pub fn info(&self, message: &str, data: Option<&str>, component: Option<&str>) {
        self.log(LogLevel::INFO, message, data, component);
    }

    pub fn warn(&self, message: &str, data: Option<&str>, component: Option<&str>) {
        self.log(LogLevel::WARN, message, data, component);
    }

    pub fn error(&self, message: &str, data: Option<&str>, component: Option<&str>) {
        self.log(LogLevel::ERROR, message, data, component);
    }

    pub fn critical(&self, message: &str, data: Option<&str>, component: Option<&str>) {
        self.log(LogLevel::CRITICAL, message, data, component);
    }

    fn log(&self, level: LogLevel, message: &str, data: Option<&str>, component: Option<&str>) {
        if level < self.log_level {
            return;
        }

        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            message: message.to_string(),
            component: component.map(|s| s.to_string()),
            data: data.map(|s| s.to_string()),
        };

        let log_line = self.format_log_entry(&entry);

        // Write to console
        println!("{}", log_line);

        // Write to file
        if let Some(log_file) = &self.log_file {
            if let Ok(mut file) = log_file.lock() {
                if let Err(e) = writeln!(file, "{}", log_line) {
                    eprintln!("Failed to write to log file: {}", e);
                }
            }
        }

        // Store in buffer
        if let Ok(mut buffer) = self.log_buffer.lock() {
            buffer.push(entry);
            if buffer.len() > self.max_buffer_size {
                buffer.remove(0);
            }
        }
    }

    fn format_log_entry(&self, entry: &LogEntry) -> String {
        let component_str = entry
            .component
            .as_ref()
            .map(|c| format!("[{}]", c))
            .unwrap_or_default();

        let data_str = entry
            .data
            .as_ref()
            .map(|d| format!(" | Data: {}", d))
            .unwrap_or_default();

        format!(
            "[{}] {} [{}] {} {} {}{}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            entry.level.emoji(),
            entry.level.as_str(),
            component_str,
            entry.message,
            data_str
        )
    }

    pub fn get_log_buffer(&self) -> Vec<LogEntry> {
        if let Ok(buffer) = self.log_buffer.lock() {
            buffer.clone()
        } else {
            Vec::new()
        }
    }

    pub fn clear_log_buffer(&self) {
        if let Ok(mut buffer) = self.log_buffer.lock() {
            buffer.clear();
        }
        self.info("Log buffer cleared", None, Some("Logger"));
    }

    pub fn export_logs(&self) -> String {
        if let Ok(buffer) = self.log_buffer.lock() {
            buffer
                .iter()
                .map(|entry| self.format_log_entry(entry))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            String::new()
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

// Global logger instance
lazy_static::lazy_static! {
    static ref LOGGER: Arc<Mutex<Logger>> = Arc::new(Mutex::new(Logger::new()));
}

// Convenience functions
pub fn log_debug(message: &str, data: Option<&str>, component: Option<&str>) {
    if let Ok(logger) = LOGGER.lock() {
        logger.debug(message, data, component);
    }
}

pub fn log_info(message: &str, data: Option<&str>, component: Option<&str>) {
    if let Ok(logger) = LOGGER.lock() {
        logger.info(message, data, component);
    }
}

pub fn log_warn(message: &str, data: Option<&str>, component: Option<&str>) {
    if let Ok(logger) = LOGGER.lock() {
        logger.warn(message, data, component);
    }
}

pub fn log_error(message: &str, data: Option<&str>, component: Option<&str>) {
    if let Ok(logger) = LOGGER.lock() {
        logger.error(message, data, component);
    }
}

pub fn log_critical(message: &str, data: Option<&str>, component: Option<&str>) {
    if let Ok(logger) = LOGGER.lock() {
        logger.critical(message, data, component);
    }
}

// Component logger creator
pub fn create_component_logger(component_name: &'static str) -> ComponentLogger {
    ComponentLogger { component_name }
}

pub struct ComponentLogger {
    component_name: &'static str,
}

impl ComponentLogger {
    pub fn debug(&self, message: &str, data: Option<&str>) {
        log_debug(message, data, Some(self.component_name));
    }

    pub fn info(&self, message: &str, data: Option<&str>) {
        log_info(message, data, Some(self.component_name));
    }

    pub fn warn(&self, message: &str, data: Option<&str>) {
        log_warn(message, data, Some(self.component_name));
    }

    pub fn error(&self, message: &str, data: Option<&str>) {
        log_error(message, data, Some(self.component_name));
    }

    pub fn critical(&self, message: &str, data: Option<&str>) {
        log_critical(message, data, Some(self.component_name));
    }
}
