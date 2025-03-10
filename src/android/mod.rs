mod steps;

#[cfg(test)]
mod mod_test;

use std::collections::HashMap;

use appium_client::capabilities::android::AndroidCapabilities;
use appium_client::find::By;
use appium_client::ClientBuilder;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::Value;


use appium_client::capabilities::{AppCapable, AppiumCapability};
use steps::execute_android_steps;

use crate::common::tags::*;
use crate::common::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AndroidStep {
    AndroidNormalStep(AndroidNormalStep),
    AndroidStepFile { step_file: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AndroidNormalStep {
    AndroidElementStep {
        selector: AndroidElementSelector,
        actions: Vec<AndroidAction>,
    },
    ScreenshotStep {
        take_screenshot: String,
    },
    LogStep {
        log: String,
    },
    Pause {
        pause: u64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AndroidAction {
    AssertVisible,
    TapOn,
    ScrollUntilVisible,
    InsertData { data: String },
    Pause(u64),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(non_snake_case)]
pub enum AndroidElementSelector {
    Hint {
        hint: String,
    },
    AccessibilityId {
        accessibilityId: String,
    },
    Text {
        text: String,
    },
    Xpath {
        xpath: String,
    },
    ClassName {
        className: String,
        instance: Option<u32>,
    },
    Id {
        id: String,
    },
    IdWithIndex {
        id: String,
        index: u32,
    },
    Description {
        description: String,
    },
    Index {
        index: u32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StepFile {
    steps: Vec<AndroidStep>,
}

pub fn set_custom_capabilities_android(
    caps: &mut AndroidCapabilities,
    custom_caps: Vec<CustomCapability>,
) {
    for custom_capability in custom_caps.clone() {
        match custom_capability.value {
            CustomCapabilityValue::BooleanValue(value) => {
                caps.set_bool(&custom_capability.key, value)
            },
            CustomCapabilityValue::StringValue(value) => {
                caps.set_str(&custom_capability.key, &value)
            },
            CustomCapabilityValue::NumberValue(value) => {
                // Handle number values appropriately
                // Convert the number to a string
                let num_str = value.to_string();
                caps.set_str(&custom_capability.key, &num_str);
            },
            CustomCapabilityValue::NullValue => {
                // Handle null values if needed
            }
        }
    }
}

pub fn get_android_element_by(selector: AndroidElementSelector) -> By {
    match selector {
        AndroidElementSelector::Index { index } => {
            By::uiautomator(&format!("new UiSelector().index({});", index))
        }
        AndroidElementSelector::AccessibilityId { accessibilityId } => {
            By::accessibility_id(&accessibilityId)
        }
        AndroidElementSelector::Xpath { xpath } => By::xpath(&xpath),
        AndroidElementSelector::Text { text } => {
            By::uiautomator(&format!("new UiSelector().textMatches(\"{}\");", text))
        }
        AndroidElementSelector::Description { description } => By::uiautomator(&format!(
            "new UiSelector().descriptionMatches(\"{}\");",
            description
        )),
        AndroidElementSelector::Hint { hint } => {
            By::xpath(&format!("//android.widget.EditText[@hint=\"{}\"]", hint))
        }
        AndroidElementSelector::IdWithIndex { id, index } => By::uiautomator(&format!(
            "new UiSelector().resourceIdMatches(\"{}\").index({});",
            id, index
        )),
        AndroidElementSelector::Id { id } => {
            By::uiautomator(&format!("new UiSelector().resourceIdMatches(\"{}\");", id))
        }
        AndroidElementSelector::ClassName {
            className,
            instance,
        } => {
            if let Some(instance) = instance {
                By::uiautomator(&format!(
                    "new UiSelector().className({}).instance({})",
                    className, instance
                ))
            } else {
                By::uiautomator(&format!("new UiSelector().className({})", className))
            }
        }
    }
}

pub async fn launch_android_main(
    capabilities: &HashMap<String, Value>,
    steps: Vec<Step>,
) -> Result<(usize, String), Box<dyn std::error::Error>> {
    // Configure the Appium driver
    let mut caps = AndroidCapabilities::new_uiautomator();

    let app_path = capabilities
        .get("appium:app")
        .expect("No app path found")
        .as_str()
        .unwrap();
    caps.app(&app_path);

    caps.platform_version(
        &capabilities
            .get("platformVersion")
            .expect("No platform version found")
            .as_str()
            .unwrap(),
    );

    for (key, value) in capabilities.iter() {
        match key.as_str() {
            "app" | "platformVersion" => continue,
            _ => match value {
                Value::String(value) => {
                    caps.set_str(key, value);
                }
                Value::Bool(value) => {
                    caps.set_bool(key, *value);
                }
                _ => {
                    eprintln!("{} Invalid value for key: {}", error_tag(), key);
                    std::process::exit(1);
                }
            },
        }
    }

    println!(
        "{} App path: {}",
        info_tag(),
        &capabilities.get("appium:app").unwrap().to_string().blue()
    );
    println!("⏳ Launching android app");
    let client = ClientBuilder::native(caps)
        .connect("http://localhost:4723/")
        .await
        .unwrap_or_else(|e| {
            println!("{} Failed to connect to Appium: {}", error_tag(), e);
            std::process::exit(1);
        });
    println!("✓ Android app launched successfully");

    let (steps_count, report) = execute_android_steps(&client, steps).await;
    Ok((steps_count, report))
}
