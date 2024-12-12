use std::{
    fs::File,
    io::{Read, Write},
};

use appium_client::{capabilities::android::AndroidCapabilities, find::By, ClientBuilder};
use serde::{Deserialize, Serialize};
use spinners::Spinner;

use appium_client::{
    capabilities::{AppCapable, AppiumCapability},
    find::AppiumFind,
    wait::AppiumWait,
};
use fantoccini::{
    actions::{InputSource, PointerAction, TouchActions, MOUSE_BUTTON_LEFT},
    client,
};
use tokio::time::{timeout, Duration};

use crate::shared::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct AndroidCaps {
    app_path: String,
    full_reset: bool,
    platform_version: String,
    custom_caps: Option<Vec<CustomCapability>>,
}

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
pub enum AndroidElementSelector {
    Text {
        text: String,
    },
    Xpath {
        xpath: String,
    },
    ClassName {
        class_name: String,
        instance: Option<i32>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StepFile {
    steps: Vec<AndroidStep>,
}

pub async fn android_transform_into_actions(steps: Vec<AndroidStep>) -> Vec<AndroidNormalStep> {
    let mut result: Vec<AndroidNormalStep> = vec![];
    for step in steps {
        match step {
            AndroidStep::AndroidNormalStep(normal_step) => result.push(normal_step),
            AndroidStep::AndroidStepFile { step_file } => {
                let mut spinner = start_spinner(format!("Loading step file: {}", step_file));
                // Load the YAML file
                let mut file = File::open(&step_file).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();

                // Deserialize the YAML into our TestFile struct
                let test_file: StepFile = serde_yaml::from_str(&contents).unwrap();
                let new_steps = Box::pin(android_transform_into_actions(test_file.steps)).await;
                for new_step in new_steps {
                    result.push(new_step)
                }
                stop_spinner(&mut spinner);
            }
        }
    }
    result
}

pub fn set_custom_capabilities_android(
    caps: &mut AndroidCapabilities,
    custom_caps: Vec<CustomCapability>,
) {
    for custom_capability in custom_caps.clone() {
        match custom_capability.value {
            CustomCapabilityValue::BooleanValue(value) => {
                caps.set_bool(&custom_capability.key, value)
            }
            CustomCapabilityValue::StringValue(value) => {
                caps.set_str(&custom_capability.key, &value)
            }
        }
    }
}

pub fn get_android_element_by(selector: AndroidElementSelector) -> By {
    match selector {
        AndroidElementSelector::Xpath { xpath } => By::xpath(&xpath),
        AndroidElementSelector::Text { text } => {
            By::uiautomator(&format!("new UiSelector().text(\"{}\");", text))
        }
        AndroidElementSelector::ClassName {
            class_name,
            instance,
        } => {
            if let Some(instance) = instance {
                By::uiautomator(&format!(
                    "new UiSelector().className({}).instance({})",
                    class_name, instance
                ))
            } else {
                By::uiautomator(&format!("new UiSelector().className({})", class_name))
            }
        }
    }
}

pub async fn launch_android_main(
    capabilities: AndroidCaps,
    steps: Vec<AndroidStep>,
) -> Result<i32, Box<dyn std::error::Error>> {
    // Flatten the steps
    let steps = android_transform_into_actions(steps).await;

    // Configure the Appium driver
    let mut caps = AndroidCapabilities::new_uiautomator();

    let app_path = capabilities.app_path.clone();
    caps.app(&app_path);

    caps.platform_version(&capabilities.platform_version);
    caps.full_reset(capabilities.full_reset);

    if let Some(custom_caps) = capabilities.custom_caps {
        set_custom_capabilities_android(&mut caps, custom_caps.clone());
    };

    println!("App path: {}", capabilities.app_path);
    let mut spinner = Spinner::new(
        spinners::Spinners::Arrow,
        "Launching android app".to_string(),
    );
    let client = ClientBuilder::native(caps)
        .connect("http://localhost:4723/")
        .await?;
    spinner.stop_with_symbol("[LAUNCHED]");

    let mut steps_count = 0;
    // Let's calculate some things first
    let (width, height) = client.get_window_size().await?;

    // This is the horizontal center, it will be our x for swipe.
    let horizontal_center = (width / 2) as i64;

    // The swipe will start at 80% of screen height, and end at 20% of screen height.
    // So we will swipe UP through most of the screen.
    let almost_top = (height as f64 * 0.2) as i64;
    let almost_bottom = (height as f64 * 0.8) as i64;
    // Process each step in the test file
    //

    for step in steps {
        match step {
            AndroidNormalStep::AndroidElementStep { selector, actions } => {
                let by = get_android_element_by(selector.clone());
                for action in actions {
                    match action {
                        AndroidAction::Pause(duration) => {
                            pause_action(duration).await;
                        }
                        AndroidAction::AssertVisible => {
                            let mut sp =
                                start_spinner(format!("Asserting visible: {:?}", selector));
                            let element =
                                client.appium_wait().for_element(by.clone()).await.unwrap();
                            let is_visible = element.is_displayed().await.unwrap();
                            assert!(is_visible);
                            steps_count += 1;
                            stop_spinner(&mut sp)
                        }
                        AndroidAction::TapOn => {
                            let mut sp = start_spinner(format!("Tapping on: {:?}", selector));
                            let element =
                                client.appium_wait().for_element(by.clone()).await.unwrap();
                            element.click().await.expect("Couldn't click on element");
                            steps_count += 1;
                            stop_spinner(&mut sp)
                        }
                        AndroidAction::InsertData { data } => {
                            let mut sp = start_spinner(format!(
                                "Inserting {} in field {:?}",
                                data, selector
                            ));
                            let element =
                                client.appium_wait().for_element(by.clone()).await.unwrap();
                            element.send_keys(&data).await.unwrap();
                            stop_spinner(&mut sp)
                        }
                        AndroidAction::ScrollUntilVisible => {
                            let mut sp =
                                start_spinner(format!("Scrolling until finding: {:?}", selector));
                            let swipe_down = TouchActions::new("finger".to_string())
                                .then(PointerAction::MoveTo {
                                    duration: Some(Duration::from_millis(0)),
                                    x: horizontal_center,
                                    y: almost_bottom,
                                })
                                .then(PointerAction::Down {
                                    button: MOUSE_BUTTON_LEFT,
                                })
                                .then(PointerAction::MoveTo {
                                    duration: Some(Duration::from_millis(250)),
                                    x: horizontal_center,
                                    y: almost_top,
                                });

                            let timeout_duration = Duration::from_secs(30);

                            let mut visible = false;

                            let _ = timeout(timeout_duration, async {
                                while !visible {
                                    if let Ok(_) = client.clone().find_by(by.clone()).await {
                                        stop_spinner(&mut sp);
                                        visible = true;
                                        steps_count += 1;
                                    } else {
                                        println!("Performing scroll");
                                        client.perform_actions(swipe_down.clone()).await.unwrap();
                                    }
                                }
                            })
                            .await;
                        }
                    }
                }
            }
            AndroidNormalStep::ScreenshotStep { take_screenshot } => {
                let mut sp = start_spinner(format!("Taking screenshot: {}", take_screenshot));
                let screenshot = client.screenshot().await.unwrap();
                let mut file = File::create(&take_screenshot).unwrap();
                file.write_all(&screenshot).unwrap();
                stop_spinner(&mut sp);
            }
            AndroidNormalStep::LogStep { log } => {
                println!("[LOG] {}", log);
            }
            AndroidNormalStep::Pause { pause } => {
                pause_action(pause).await;
            }
        }
    }
    Ok(steps_count)
}
