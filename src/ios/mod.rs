use std::{
    fs::File,
    i64,
    io::{Read, Write},
};

use serde::{Deserialize, Serialize};

use appium_client::{
    capabilities::{ios::IOSCapabilities, AppCapable, AppiumCapability},
    find::{AppiumFind, By},
    wait::AppiumWait,
    ClientBuilder,
};
use fantoccini::{
    actions::{InputSource, PointerAction, TouchActions, MOUSE_BUTTON_LEFT},
    Client,
};
use spinners::Spinner;
use tokio::time::{timeout, Duration};

use crate::common::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct IosCaps {
    app_path: String,
    custom_caps: Option<Vec<CustomCapability>>,
    device_name: String,
    platform_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IosElementSelector {
    Xpath(String),
    ClassName(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IosAction {
    AssertVisible,
    TapOn,
    ScrollUntilVisible,
    InsertData { data: String },
    Pause(u64),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IosStep {
    IosNormalStep(IosNormalStep),
    IosStepFile { step_file: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StepFile {
    steps: Vec<IosStep>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IosNormalStep {
    IosElementStep {
        selector: IosElementSelector,
        actions: Vec<IosAction>,
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
pub fn set_custom_capabilities_ios(caps: &mut IOSCapabilities, custom_caps: Vec<CustomCapability>) {
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

pub fn get_ios_element_by(selector: IosElementSelector) -> By {
    match selector {
        IosElementSelector::Xpath(xpath) => By::xpath(&xpath),
        IosElementSelector::ClassName(class_name) => By::class_name(&class_name),
    }
}

pub async fn launch_ios_main(
    capabilities: IosCaps,
    steps: Vec<IosStep>,
) -> Result<i32, Box<dyn std::error::Error>> {
    let steps = ios_transform_into_actions(steps).await;
    // Configure the Appium driver
    let mut caps = IOSCapabilities::new_xcui();

    let app_path = capabilities.app_path.clone();
    caps.app(&app_path);

    caps.device_name(&capabilities.device_name);
    caps.platform_version(&capabilities.platform_version);
    caps.automation_name("XCUITest");

    if let Some(custom_caps) = capabilities.custom_caps {
        set_custom_capabilities_ios(&mut caps, custom_caps)
    };

    println!("App path: {}", capabilities.app_path);
    let mut spinner = Spinner::new(spinners::Spinners::Arrow, "Launching ios app".to_string());
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
            IosNormalStep::ScreenshotStep { take_screenshot } => {
                let mut sp = start_spinner(format!("Taking screenshot: {}", take_screenshot));
                let screenshot = client.screenshot().await.unwrap();
                let mut file = File::create(&take_screenshot).unwrap();
                file.write_all(&screenshot).unwrap();
                stop_spinner(&mut sp);
            }
            IosNormalStep::LogStep { log } => {
                println!("[LOG] {}", log);
            }
            IosNormalStep::IosElementStep { selector, actions } => {
                steps_count += ios_perform_actions(
                    client.clone(),
                    selector,
                    actions,
                    horizontal_center,
                    almost_top,
                    almost_bottom,
                )
                .await;
            }
            IosNormalStep::Pause { pause } => {
                pause_action(pause).await;
            }
        }
    }

    Ok(steps_count)
}

pub async fn ios_perform_actions(
    client: Client,
    selector: IosElementSelector,
    actions: Vec<IosAction>,
    horizontal_center: i64,
    almost_top: i64,
    almost_bottom: i64,
) -> i32 {
    let by = get_ios_element_by(selector.clone());
    let mut steps_count: i32 = 0;
    for action in actions {
        match action {
            IosAction::Pause(duration) => {
                pause_action(duration).await;
            }
            IosAction::AssertVisible => {
                let mut sp = start_spinner(format!("Asserting visible: {:?}", selector));
                let element = client.appium_wait().for_element(by.clone()).await.unwrap();
                let is_visible = element.is_displayed().await.unwrap();
                assert!(is_visible);
                steps_count += 1;
                stop_spinner(&mut sp)
            }
            IosAction::TapOn => {
                let mut sp = start_spinner(format!("Tapping on: {:?}", selector));
                let element = client.appium_wait().for_element(by.clone()).await.unwrap();
                element.click().await.expect("Couldn't click on element");
                steps_count += 1;
                stop_spinner(&mut sp)
            }
            IosAction::InsertData { data } => {
                let mut sp = start_spinner(format!("Inserting {} in field {:?}", data, selector));
                let element = client.appium_wait().for_element(by.clone()).await.unwrap();
                element.send_keys(&data).await.unwrap();
                stop_spinner(&mut sp)
            }
            IosAction::ScrollUntilVisible => {
                let mut sp = start_spinner(format!("Scrolling until finding: {:?}", selector));
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
    steps_count
}

pub async fn ios_transform_into_actions(steps: Vec<IosStep>) -> Vec<IosNormalStep> {
    let mut result: Vec<IosNormalStep> = vec![];
    for step in steps {
        match step {
            IosStep::IosNormalStep(normal_step) => result.push(normal_step),
            IosStep::IosStepFile { step_file } => {
                let mut spinner = start_spinner(format!("Loading step file: {}", step_file));
                // Load the YAML file
                let mut file = File::open(&step_file).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();

                // Deserialize the YAML into our TestFile struct
                let test_file: StepFile = serde_yaml::from_str(&contents).unwrap();
                let new_steps = Box::pin(ios_transform_into_actions(test_file.steps)).await;
                for new_step in new_steps {
                    result.push(new_step)
                }
                stop_spinner(&mut spinner);
            }
        }
    }
    result
}
