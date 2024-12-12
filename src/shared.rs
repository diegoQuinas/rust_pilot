use std::time::Duration;

use serde::{Deserialize, Serialize};
use spinners::Spinner;
use tokio::time::sleep;

use crate::android::*;
use crate::ios::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestFile {
    pub platform: Platform,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Android {
        capabilities: AndroidCaps,
        steps: Vec<AndroidStep>,
    },
    Ios {
        capabilities: IosCaps,
        steps: Vec<IosStep>,
    },
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
