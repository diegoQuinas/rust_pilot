use appium_client::{
    capabilities::{
        android::AndroidCapabilities, ios::IOSCapabilities, AppCapable, AppiumCapability,
    },
    find::{AppiumFind, By},
    wait::AppiumWait,
    ClientBuilder,
};
use fantoccini::actions::{InputSource, PointerAction, TouchActions, MOUSE_BUTTON_LEFT};
use serde::{Deserialize, Serialize};
use spinners::Spinner;
use std::{env, fs::File, io::Read, time::Instant};
use tokio::time::{timeout, Duration};

#[derive(Debug, Serialize, Deserialize)]
struct TestFile {
    platform: Platform,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Platform {
    Android {
        capabilities: AndroidCaps,
        steps: Vec<AndroidStep>,
    },
    Ios {
        capabilities: IosCaps,
        steps: Vec<IosStep>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct IosCaps {
    app_path: String,
    custom_caps: Option<Vec<CustomCapability>>,
    device_name: String,
    platform_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct IosStep {
    selector: IosElementSelector,
    actions: Vec<IosAction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum IosElementSelector {
    Xpath(String),
    ClassName(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum IosAction {
    AssertVisible,
    TapOn,
    ScrollUntilVisible,
    InsertData { data: String },
}

#[derive(Debug, Serialize, Deserialize)]
struct AndroidCaps {
    app_path: String,
    full_reset: bool,
    platform_version: String,
    custom_caps: Option<Vec<CustomCapability>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CustomCapability {
    key: String,
    value: CustomCapabilityValue,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum CustomCapabilityValue {
    BooleanValue(bool),
    StringValue(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct AndroidStep {
    selector: AndroidElementSelector,
    actions: Vec<AndroidAction>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AndroidAction {
    AssertVisible,
    TapOn,
    ScrollUntilVisible,
    InsertData { data: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum AndroidElementSelector {
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

fn set_custom_capabilities_ios(caps: &mut IOSCapabilities, custom_caps: Vec<CustomCapability>) {
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

fn set_custom_capabilities_android(
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

fn get_ios_element_by(selector: IosElementSelector) -> By {
    match selector {
        IosElementSelector::Xpath(xpath) => By::xpath(&xpath),
        IosElementSelector::ClassName(class_name) => By::class_name(&class_name),
    }
}
fn get_android_element_by(selector: AndroidElementSelector) -> By {
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

fn start_spinner(message: String) -> Spinner {
    Spinner::new(spinners::Spinners::Dots, message)
}

fn stop_spinner(spinner: &mut Spinner) {
    spinner.stop_with_symbol("\x1b[32m[OK]\x1b[0m")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Obtener los argumentos de la línea de comandos
    let args: Vec<String> = env::args().collect();

    // Verificar si el archivo fue pasado como argumento
    if args.len() < 2 {
        println!("Test file path needed as argument");
        return Ok(());
    }

    let file_path = &args[1];

    println!("Test file path: {}", file_path);
    // Load the YAML file
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Deserialize the YAML into our TestFile struct
    let test_file: TestFile = serde_yaml::from_str(&contents)?;

    let start = Instant::now();
    let steps_count = match test_file.platform {
        Platform::Ios {
            capabilities,
            steps,
        } => launch_ios_main(capabilities, steps).await.unwrap(),
        Platform::Android {
            capabilities,
            steps,
        } => launch_android_main(capabilities, steps).await.unwrap(),
    };
    let time = start.elapsed();
    println!("\n\nTest suite runned successfully");
    println!("    Actions executed: {}", steps_count);
    println!("    Total time elapsed: {:.2} seconds", time.as_secs_f64());
    Ok(())
}

async fn launch_android_main(
    capabilities: AndroidCaps,
    steps: Vec<AndroidStep>,
) -> Result<i32, Box<dyn std::error::Error>> {
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
        let by = get_android_element_by(step.selector.clone());
        for action in step.actions {
            match action {
                AndroidAction::AssertVisible => {
                    let mut sp = start_spinner(format!("Asserting visible: {:?}", step.selector));
                    let element = client.appium_wait().for_element(by.clone()).await.unwrap();
                    let is_visible = element.is_displayed().await.unwrap();
                    assert!(is_visible);
                    steps_count += 1;
                    stop_spinner(&mut sp)
                }
                AndroidAction::TapOn => {
                    let mut sp = start_spinner(format!("Tapping on: {:?}", step.selector));
                    let element = client.appium_wait().for_element(by.clone()).await.unwrap();
                    element.click().await.expect("Couldn't click on element");
                    steps_count += 1;
                    stop_spinner(&mut sp)
                }
                AndroidAction::InsertData { data } => {
                    let mut sp =
                        start_spinner(format!("Inserting {} in field {:?}", data, step.selector));
                    let element = client.appium_wait().for_element(by.clone()).await.unwrap();
                    element.send_keys(&data).await.unwrap();
                    stop_spinner(&mut sp)
                }
                AndroidAction::ScrollUntilVisible => {
                    let mut sp =
                        start_spinner(format!("Scrolling until finding: {:?}", step.selector));
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

    Ok(steps_count)
}

async fn launch_ios_main(
    capabilities: IosCaps,
    steps: Vec<IosStep>,
) -> Result<i32, Box<dyn std::error::Error>> {
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
        let by = get_ios_element_by(step.selector.clone());
        for action in step.actions {
            match action {
                IosAction::AssertVisible => {
                    let mut sp = start_spinner(format!("Asserting visible: {:?}", step.selector));
                    let element = client.appium_wait().for_element(by.clone()).await.unwrap();
                    let is_visible = element.is_displayed().await.unwrap();
                    assert!(is_visible);
                    steps_count += 1;
                    stop_spinner(&mut sp)
                }
                IosAction::TapOn => {
                    let mut sp = start_spinner(format!("Tapping on: {:?}", step.selector));
                    let element = client.appium_wait().for_element(by.clone()).await.unwrap();
                    element.click().await.expect("Couldn't click on element");
                    steps_count += 1;
                    stop_spinner(&mut sp)
                }
                IosAction::InsertData { data } => {
                    let mut sp =
                        start_spinner(format!("Inserting {} in field {:?}", data, step.selector));
                    let element = client.appium_wait().for_element(by.clone()).await.unwrap();
                    element.send_keys(&data).await.unwrap();
                    stop_spinner(&mut sp)
                }
                IosAction::ScrollUntilVisible => {
                    let mut sp =
                        start_spinner(format!("Scrolling until finding: {:?}", step.selector));
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

    Ok(steps_count)
}
