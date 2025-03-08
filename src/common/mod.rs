use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process;
use std::time::Duration;
use std::cell::RefCell;

use colored::Colorize;
use fantoccini::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_yaml;
use serde_yaml::Deserializer;

use tokio::time::sleep;

pub mod tags;
use tags::*;
use crate::logger::Logger;

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct TestFileHeader {
    pub appId: Option<String>,
    pub tags: Option<Vec<String>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CapsFile {
    pub platform: Platform,
    pub app_path: String,
    pub full_reset: bool,
    pub platform_version: String,
    pub custom_caps: Option<Vec<CustomCapability>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StepFile(pub Vec<Step>);

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
#[allow(non_snake_case)]
pub enum Step {
    RunFlow { runFlow: String },
    TapOn { tapOn: TapOn },
    RunScript { runScript: String },
    InputText { inputText: String },
    AssertVisible { assertVisible: String },
    AssertNotVisible { assertNotVisible: String },
    LaunchApp { launchApp: LaunchApp },
    Swipe { swipe: SwipeOptions },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SwipeOptions {
    pub start: ScreenPercentages,
    pub end: ScreenPercentages,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScreenPercentages(String);

impl ScreenPercentages {
    pub fn to_f64(&self) -> (f64, f64) {
        let numbers: Vec<f64> = self
            .0
            .split(",")
            .map(|s| s.trim_end_matches("%"))
            .map(|s| s.trim())
            .map(|number| {
                number.parse::<f64>().unwrap_or_else(|err| {
                    eprintln!(
                        "{} Error: Swipe percentage must be a number: {:#?}, {}",
                        error_tag(),
                        number,
                        err
                    );
                    process::exit(1);
                })
            })
            .collect();

        if numbers.len() != 2 {
            eprintln!(
                "{} Error: Swipe percentage must have two values: x and y",
                error_tag()
            );
            process::exit(1);
        }

        for number in numbers.clone() {
            if number > 100.0 || number < 0.0 {
                eprintln!(
                    "{} Error: Swipe percentage must be between 0 and 100",
                    error_tag()
                );
                process::exit(1);
            }
        }
        (numbers[0], numbers[1])
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct LaunchApp {
    pub clearState: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum TapOn {
    TapOnTextOrDescription(String),
    TapOnOption(TapOnOption),
}
#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct TapOnOption {
    pub id: Option<String>,
    pub text: Option<String>,
    pub optional: Option<bool>,
    pub index: Option<u32>,
    pub instance: Option<u32>,
    pub className: Option<String>,
    pub description: Option<String>,
    pub hint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Android,
    Ios,
    Flutter,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SharedAction {
    Pause { duration: u64 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomCapability {
    pub key: String,
    pub value: CustomCapabilityValue,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CustomCapabilityValue {
    BooleanValue(bool),
    StringValue(String),
}

pub async fn pause_action(duration: u64) {
    println!("⏳ Pausing for {} millis", duration);
    let duration = Duration::from_millis(duration);
    sleep(duration).await;
    println!("✓ Pause completed");
}

// Plain text logger instead of spinner
pub struct PlainLogger {
    pub message: String,
    pub indent_level: usize,
}

// Global indentation level for nested steps
thread_local! {
    static CURRENT_INDENT_LEVEL: RefCell<usize> = RefCell::new(0);
}

pub fn get_current_indent_level() -> usize {
    CURRENT_INDENT_LEVEL.with(|level| *level.borrow())
}

pub fn set_current_indent_level(level: usize) {
    CURRENT_INDENT_LEVEL.with(|current_level| *current_level.borrow_mut() = level);
}

pub fn start_spinner(message: String) -> PlainLogger {
    let indent_level = get_current_indent_level();
    if indent_level > 0 {
        crate::logger::Logger::step_with_indent(message.clone(), indent_level);
    } else {
        crate::logger::Logger::step(message.clone());
    }
    PlainLogger { message, indent_level }
}

pub fn stop_spinner(logger: &mut PlainLogger) {
    let message = format!("{} - Completed", logger.message);
    if logger.indent_level > 0 {
        crate::logger::Logger::success_with_indent(message, logger.indent_level);
    } else {
        crate::logger::Logger::success(message);
    }
}

impl PlainLogger {
    pub fn stop_with_symbol(&self, message: &str) {
        if self.indent_level > 0 {
            crate::logger::Logger::info_with_indent(message, self.indent_level);
        } else {
            crate::logger::Logger::info(message);
        }
    }
}

/// Flattens a list of steps, resolving any `RunFlow` steps recursively.
pub async fn flatten_steps(
    steps: Vec<Step>,
    base_path: &Path,
    mermaid_parent_id: String,
) -> (Vec<Step>, String) {
    // Call the internal function with indent level 0
    flatten_steps_with_indent(steps, base_path, mermaid_parent_id, 0).await
}

/// Internal implementation of flatten_steps that tracks the indentation level
async fn flatten_steps_with_indent(
    steps: Vec<Step>,
    base_path: &Path,
    mermaid_parent_id: String,
    indent_level: usize,
) -> (Vec<Step>, String) {
    let mut flattened_steps: Vec<Step> = Vec::new();
    let mut mermaid_steps = String::new();

    for step in steps {
        match step {
            Step::RunFlow { runFlow } => {
                let now = chrono::Local::now().timestamp_millis();
                let id = format!("idRunFlow{}({})", now, runFlow);
                mermaid_steps.push_str(&format!("{} --> {}\n", mermaid_parent_id, id));
                let step_path = base_path.join(&runFlow);
                let string_path = step_path.display().to_string();
                
                // Use indented logging
                Logger::info_with_indent(format!("Loading step file {}", string_path.blue()), indent_level);

                // Verificar existencia del archivo
                if !step_path.exists() {
                    Logger::error_with_indent(
                        format!("Error: File {} does not exist", step_path.display()),
                        indent_level
                    );
                    process::exit(1);
                }

                // Parsear el archivo de pasos
                let (_, steps) = parse_test_file(&step_path);

                // Store the current indentation level for the nested steps
                let next_indent_level = indent_level + 1;
                
                // Recursivamente aplanar los pasos del archivo cargado con un nivel más de indentación
                let (sub_steps, mermaid_sub_steps) =
                    Box::pin(flatten_steps_with_indent(
                        steps, 
                        step_path.parent().unwrap(), 
                        id,
                        next_indent_level // Increase indent level for nested steps
                    )).await;
                
                // Add the indentation level to each step
                let mut indented_steps: Vec<Step> = Vec::new();
                for step in sub_steps {
                    // Store the indentation level in the step metadata
                    let step_with_indent = match step {
                        Step::RunFlow { runFlow } => Step::RunFlow { runFlow },
                        Step::TapOn { tapOn } => {
                            // Set the indentation level for this step
                            set_current_indent_level(next_indent_level);
                            Step::TapOn { tapOn }
                        },
                        Step::RunScript { runScript } => {
                            set_current_indent_level(next_indent_level);
                            Step::RunScript { runScript }
                        },
                        Step::InputText { inputText } => {
                            set_current_indent_level(next_indent_level);
                            Step::InputText { inputText }
                        },
                        Step::AssertVisible { assertVisible } => {
                            set_current_indent_level(next_indent_level);
                            Step::AssertVisible { assertVisible }
                        },
                        Step::AssertNotVisible { assertNotVisible } => {
                            set_current_indent_level(next_indent_level);
                            Step::AssertNotVisible { assertNotVisible }
                        },
                        Step::LaunchApp { launchApp } => {
                            set_current_indent_level(next_indent_level);
                            Step::LaunchApp { launchApp }
                        },
                        Step::Swipe { swipe } => {
                            set_current_indent_level(next_indent_level);
                            Step::Swipe { swipe }
                        },
                    };
                    indented_steps.push(step_with_indent);
                }
                
                flattened_steps.extend(indented_steps);

                let string_path = step_path.display().to_string();
                Logger::success_with_indent(
                    format!("Steps from {} loaded successfully", string_path.blue()),
                    indent_level
                );
                mermaid_steps.push_str(&mermaid_sub_steps);
            }
            step => {
                let now = chrono::Local::now().timestamp_millis();
                let step_name = format!("{:?}", step);
                let step_name: String = step_name.split_whitespace().next().unwrap().to_string();
                let node = format!("idStepName{}({})", now, step_name);
                mermaid_steps.push_str(&format!("{} --> {}\n", mermaid_parent_id, node));
                
                // For top-level steps (indent_level == 0), we need to set the indentation level to 0
                // For nested steps, the indentation level is already set in the RunFlow branch
                if indent_level == 0 {
                    set_current_indent_level(0);
                } else {
                    set_current_indent_level(indent_level);
                }
                
                flattened_steps.push(step);
            }
        }
    }

    (flattened_steps, mermaid_steps)
}

pub async fn error_take_screenshot(client: &Client) {
    // Set indentation level to 0 for error screenshots
    set_current_indent_level(0);
    crate::logger::Logger::error("Taking error screenshot");
    take_screenshot(client, "error_screenshot.png").await;
}

pub async fn take_screenshot(client: &Client, take_screenshot: &str) {
    // Use the Logger for consistent formatting
    let indent_level = get_current_indent_level();
    crate::logger::Logger::step_with_indent(format!("Taking screenshot: {}", take_screenshot), indent_level);
    
    let screenshot = client.screenshot().await.unwrap();
    let mut file = File::create(&take_screenshot).unwrap();
    file.write_all(&screenshot).unwrap();
    
    crate::logger::Logger::success_with_indent("Screenshot taken", indent_level);
}

pub fn parse_test_file<P: AsRef<Path>>(path: P) -> (TestFileHeader, Vec<Step>) {
    let content = get_content(&path);
    let (header, steps) = deserialize_test_file(&content);
    (header, steps)
}

fn get_content<P: AsRef<Path>>(path: P) -> String {
    let content = fs::read_to_string(&path).unwrap_or_else(|err| {
        eprintln!(
            "{} Failed to read YAML file {}: {}",
            error_tag(),
            path.as_ref().display().to_string().blue(),
            err.to_string().red()
        );
        process::exit(1);
    });
    content
}

fn deserialize_test_file(content: &str) -> (TestFileHeader, Vec<Step>) {
    let mut documents = Deserializer::from_str(&content);

    let header: TestFileHeader = deserialize_document(documents.next(), "header");

    let steps: StepFile = deserialize_document(documents.next(), "steps");

    (header, steps.0)
}

fn deserialize_document<T: DeserializeOwned>(
    deserializer: Option<Deserializer>,
    context: &str,
) -> T {
    match deserializer {
        Some(deserializer) => T::deserialize(deserializer).unwrap_or_else(|err| {
            eprintln!("{} Error deserializing {}: {}", error_tag(), context, err);
            process::exit(1);
        }),
        None => {
            eprintln!("{} Missing {} document in YAML file", error_tag(), context);
            process::exit(1);
        }
    }
}
