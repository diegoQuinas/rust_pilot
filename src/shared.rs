use std::process;
use std::time::Duration;
use std::{fs, path::Path};

use colored::Colorize;
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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LaunchApp {
    pub clearState: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum TapOn {
    TapOnText(String),
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
    stop_spinner(&mut sp);
}

pub fn start_spinner(message: String) -> Spinner {
    Spinner::new(spinners::Spinners::Dots, message)
}

pub fn stop_spinner(spinner: &mut Spinner) {
    spinner.stop_with_symbol("\x1b[32m[OK]\x1b[0m")
}

/// Flattens a list of steps, resolving any `RunFlow` steps recursively.
pub async fn flatten_steps(steps: Vec<Step>, base_path: &Path) -> Vec<Step> {
    let mut flattened_steps: Vec<Step> = Vec::new();

    for step in steps {
        match step {
            Step::RunFlow { runFlow } => {
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
                let sub_steps = Box::pin(flatten_steps(steps, step_path.parent().unwrap())).await;
                flattened_steps.extend(sub_steps);

                let string_path = step_path.display().to_string();
                println!(
                    "{} Steps from {} loaded successfully",
                    ok_tag(),
                    string_path.blue()
                );
            }
            _ => {
                // Agregar cualquier otro paso directamente al resultado
                flattened_steps.push(step);
            }
        }
    }

    flattened_steps
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
