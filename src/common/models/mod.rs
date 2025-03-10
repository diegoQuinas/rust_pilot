use std::process;
use serde::{Deserialize, Serialize};
use crate::common::tags::error_tag;

/// Header information for test files
#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct TestFileHeader {
    pub appId: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Capabilities file structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CapsFile {
    pub platform: Platform,
    pub app_path: String,
    pub full_reset: bool,
    pub platform_version: String,
    pub custom_caps: Option<Vec<CustomCapability>>,
}

/// Step file container
#[derive(Debug, Serialize, Deserialize)]
pub struct StepFile(pub Vec<Step>);

/// Represents a test step
#[derive(Serialize, Deserialize, Debug, Clone)]
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

/// Swipe options for gesture actions
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SwipeOptions {
    pub start: ScreenPercentages,
    pub end: ScreenPercentages,
}

/// Screen percentages for positioning
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScreenPercentages(pub String);

impl ScreenPercentages {
    /// Converts percentage string to float coordinates
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

/// Launch app configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct LaunchApp {
    pub clearState: bool,
}

/// TapOn action variants
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum TapOn {
    TapOnTextOrDescription(String),
    TapOnOption(TapOnOption),
}

/// Tap options for element selection
#[derive(Serialize, Deserialize, Debug, Clone)]
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

/// Supported platforms
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Android,
    Ios,
    Flutter,
}

/// Shared actions across platforms
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SharedAction {
    Pause { duration: u64 },
}

/// Custom capability for device configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomCapability {
    pub key: String,
    pub value: CustomCapabilityValue,
}

/// Custom capability value types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CustomCapabilityValue {
    BooleanValue(bool),
    StringValue(String),
    NumberValue(f64),
    NullValue,
}
