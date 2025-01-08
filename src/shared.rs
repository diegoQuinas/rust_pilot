use std::any::type_name;
use std::fs::File;
use std::io::Write;
use std::process;
use std::time::Duration;
use std::{fs, path::Path};

use colored::Colorize;
use fantoccini::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_yaml::Deserializer;
use serde_yaml::{self, Value};
use spinners::Spinner;
use tokio::time::sleep;

use crate::tags::{error_tag, info_tag, ok_tag};

#[derive(Debug, Serialize, Deserialize)]
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
        let numbers = self
            .0
            .split(",")
            .map(|s| s.trim_end_matches("%"))
            .map(|s| s.trim())
            .map(|number| {
                number.parse::<f64>().unwrap_or_else(|err| {
                    eprintln!(
                        "{} Error: Swipe percentage must be a number: {:?}, {}",
                        error_tag(),
                        number,
                        err
                    );
                    process::exit(1);
                })
            })
            .collect::<Vec<f64>>();

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
pub struct TapOnOption {
    pub id: Option<String>,
    pub text: Option<String>,
    pub optional: Option<bool>,
    pub index: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Android,
    Ios,
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
    let mut sp = start_spinner(format!("Pausing for {duration} millis"));
    let duration = Duration::from_millis(duration);
    sleep(duration).await;
}

pub fn start_spinner(message: String) -> Spinner {
    Spinner::new(spinners::Spinners::Dots, message)
}

/// Flattens a list of steps, resolving any `RunFlow` steps recursively.
pub async fn flatten_steps(
    steps: Vec<Step>,
    base_path: &Path,
    mermaid_parent_id: String,
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
                println!("{} Loading step file {}", info_tag(), string_path.blue());

                // Verificar existencia del archivo
                if !step_path.exists() {
                    eprintln!(
                        "{} Error: File {} does not exist",
                        error_tag(),
                        step_path.display()
                    );
                    process::exit(1);
                }

                // Parsear el archivo de pasos
                let (_, steps) = parse_test_file(step_path.clone());

                // Recursivamente aplanar los pasos del archivo cargado
                let (sub_steps, mermaid_sub_steps) =
                    Box::pin(flatten_steps(steps, step_path.parent().unwrap(), id)).await;
                flattened_steps.extend(sub_steps);

                let string_path = step_path.display().to_string();
                println!(
                    "{} Steps from {} loaded successfully",
                    ok_tag(),
                    string_path.blue()
                );
                mermaid_steps.push_str(&mermaid_sub_steps);
            }
            step => {
                let now = chrono::Local::now().timestamp_millis();
                let step_name = format!("{:?}", step);
                let step_name: String = step_name.split_whitespace().next().unwrap().to_string();
                let node = format!("idStepName{}({})", now, step_name);
                mermaid_steps.push_str(&format!("{} --> {}\n", mermaid_parent_id, node));
                flattened_steps.push(step);
            }
        }
    }

    (flattened_steps, mermaid_steps)
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

/// Función principal para parsear el archivo de prueba
pub fn parse_test_file<P: AsRef<Path>>(path: P) -> (TestFileHeader, Vec<Step>) {
    let content = fs::read_to_string(&path).unwrap_or_else(|err| {
        eprintln!(
            "{} Failed to read YAML file {}: {}",
            error_tag(),
            path.as_ref().display(),
            err
        );
        process::exit(1);
    });

    // Crear el deserializador para múltiples documentos
    let mut documents = Deserializer::from_str(&content);

    // Deserializar el encabezado
    let header: TestFileHeader = deserialize_document(documents.next(), "header");

    // Deserializar los pasos
    let steps: StepFile = deserialize_document(documents.next(), "steps");

    (header, steps.0)
}

pub async fn error_take_screenshot(client: &Client) {
    take_screenshot(client, "error_screenshot.png").await;
}

pub async fn take_screenshot(client: &Client, take_screenshot: &str) {
    let mut sp = start_spinner(format!("Taking screenshot: {}", take_screenshot));
    let screenshot = client.screenshot().await.unwrap();
    let mut file = File::create(&take_screenshot).unwrap();
    file.write_all(&screenshot).unwrap();
    sp.stop_with_symbol(&format!("{} Screenshot taken", ok_tag()));
}
