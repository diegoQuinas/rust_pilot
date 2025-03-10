//! Utility functions for the common module

use std::cell::RefCell;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process;
use std::time::Duration;

// Removed unused import: colored::Colorize

use fantoccini::Client as FantoClient;
use serde::de::DeserializeOwned;
use serde_yaml;
use serde_yaml::Deserializer;
use tokio::time::sleep;

use crate::common::models::{Step, TestFileHeader};
use crate::logger::Logger;

// Thread-local storage for indentation level
thread_local! {
    static CURRENT_INDENT_LEVEL: RefCell<usize> = RefCell::new(0);
}

/// Get the current indentation level
pub fn get_current_indent_level() -> usize {
    CURRENT_INDENT_LEVEL.with(|level| *level.borrow())
}

/// Set the current indentation level
pub fn set_current_indent_level(level: usize) {
    CURRENT_INDENT_LEVEL.with(|current_level| *current_level.borrow_mut() = level);
}

/// Plain text logger for step execution
pub struct PlainLogger {
    pub message: String,
    pub indent_level: usize,
}

/// Create a spinner with a message
pub fn start_spinner(message: String) -> PlainLogger {
    let indent_level = get_current_indent_level();
    if indent_level > 0 {
        Logger::step_with_indent(message.clone(), indent_level);
    } else {
        Logger::step(message.clone());
    }
    PlainLogger {
        message,
        indent_level,
    }
}

/// Stop a spinner with completion message
pub fn stop_spinner(logger: &mut PlainLogger) {
    let message = format!("{} - Completed", logger.message);
    if logger.indent_level > 0 {
        Logger::success_with_indent(message, logger.indent_level);
    } else {
        Logger::success(message);
    }
}

impl PlainLogger {
    /// Stop with a custom message
    pub fn stop_with_symbol(&self, message: &str) {
        if self.indent_level > 0 {
            Logger::info_with_indent(message, self.indent_level);
        } else {
            Logger::info(message);
        }
    }
}

/// Pause execution for the specified duration
pub async fn pause_action(duration: u64) {
    let indent_level = get_current_indent_level();
    if indent_level > 0 {
        Logger::info_with_indent(format!("Pausing for {} ms", duration), indent_level);
    } else {
        Logger::info(format!("Pausing for {} ms", duration));
    }

    sleep(Duration::from_millis(duration)).await;

    if indent_level > 0 {
        Logger::success_with_indent("Pause completed", indent_level);
    } else {
        Logger::success("Pause completed");
    }
}

/// Take a screenshot on error
pub async fn error_take_screenshot(client: &FantoClient) {
    // Set indentation level to 0 for error screenshots
    set_current_indent_level(0);
    Logger::info("Taking error screenshot");
    take_screenshot(client, "error_screenshot.png").await;
}

/// Take a screenshot with the specified filename
pub async fn take_screenshot(client: &FantoClient, take_screenshot: &str) {
    // Use the Logger for consistent formatting
    let indent_level = get_current_indent_level();
    Logger::step_with_indent(
        format!("Taking screenshot: {}", take_screenshot),
        indent_level,
    );

    let screenshot = client.screenshot().await.unwrap();
    let mut file = File::create(&take_screenshot).unwrap();
    file.write_all(&screenshot).unwrap();

    Logger::success_with_indent("Screenshot taken", indent_level);
}

/// Parse a test file and return its header and steps
pub fn parse_test_file<P: AsRef<Path>>(path: P) -> (TestFileHeader, Vec<Step>) {
    let content = get_content(&path);
    let (header, steps) = deserialize_test_file(&content);
    (header, steps)
}

/// Read content from a file
pub fn get_content<P: AsRef<Path>>(path: P) -> String {
    match fs::read_to_string(path.as_ref()) {
        Ok(content) => content,
        Err(e) => {
            Logger::error(format!(
                "Error reading file {}: {}",
                path.as_ref().display(),
                e
            ));
            process::exit(1);
        }
    }
}

/// Deserialize a test file from YAML content
pub fn deserialize_test_file(content: &str) -> (TestFileHeader, Vec<Step>) {
    let mut deserializer = Deserializer::from_str(content);
    let header: TestFileHeader = deserialize_document(deserializer.next(), "header");
    let steps: Vec<Step> = deserialize_document(deserializer.next(), "steps");
    (header, steps)
}

/// Deserialize a document from YAML
pub fn deserialize_document<T: DeserializeOwned>(
    deserializer: Option<Deserializer>,
    context: &str,
) -> T {
    match deserializer {
        Some(d) => match T::deserialize(d) {
            Ok(doc) => doc,
            Err(e) => {
                Logger::error(format!("Error deserializing {}: {}", context, e));
                process::exit(1);
            }
        },
        None => {
            Logger::error(format!("Missing {} in YAML file", context));
            process::exit(1);
        }
    }
}
