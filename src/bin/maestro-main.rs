use appium_client::{
    capabilities::{android::AndroidCapabilities, AppCapable, AppiumCapability},
    find::{AppiumFind, By},
    wait::AppiumWait,
    ClientBuilder,
};
use fantoccini::actions::{InputSource, PointerAction, TouchActions, MOUSE_BUTTON_LEFT};
use serde::{Deserialize, Serialize};
use std::{env, fs::File, io::Read, time::Instant};
use tokio::time::{timeout, Duration};

#[derive(Debug, Serialize, Deserialize)]
struct TestFile {
    capabilities: TestCapabilities,
    steps: Vec<Step>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestCapabilities {
    app_path: String,
    full_reset: bool,
    platform_version: String,
    custom_cap: Vec<CustomCapability>,
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
struct Step {
    selector: ElementSelector,
    actions: Vec<Action>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Action {
    AssertVisible,
    TapOn,
    ScrollUntilVisible,
    InsertData { data: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ElementSelector {
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

fn set_custom_capabilities(caps: &mut AndroidCapabilities, test_file: &TestFile) {
    for custom_capability in test_file.capabilities.custom_cap.clone() {
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

fn get_element_by(selector: ElementSelector) -> By {
    match selector {
        ElementSelector::Xpath { xpath } => By::xpath(&xpath),
        ElementSelector::Text { text } => {
            By::uiautomator(&format!("new UiSelector().text(\"{}\");", text))
        }
        ElementSelector::ClassName {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Obtener los argumentos de la línea de comandos
    let args: Vec<String> = env::args().collect();

    // Verificar si el archivo fue pasado como argumento
    if args.len() < 2 {
        println!("File path needed as argument");
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

    // Configure the Appium driver
    let mut caps = AndroidCapabilities::new_uiautomator();

    let app_path = test_file.capabilities.app_path.clone();
    caps.app(&app_path);

    caps.platform_version(&test_file.capabilities.platform_version);
    caps.full_reset(test_file.capabilities.full_reset);

    set_custom_capabilities(&mut caps, &test_file);

    println!("Launching app {}", test_file.capabilities.app_path);
    let client = ClientBuilder::native(caps)
        .connect("http://localhost:4723/")
        .await?;
    println!("Launched!");

    let start = Instant::now();
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

    for step in test_file.steps {
        let by = get_element_by(step.selector.clone());
        for action in step.actions {
            match action {
                Action::AssertVisible => {
                    println!("Asserting visible: {:?}", step.selector);
                    let element = client.appium_wait().for_element(by.clone()).await.unwrap();
                    let is_visible = element.is_displayed().await.unwrap();
                    assert!(is_visible);
                    println!("Visible!");
                    steps_count += 1;
                }
                Action::TapOn => {
                    println!("Tapping on: {:?}", step.selector);
                    let element = client.appium_wait().for_element(by.clone()).await.unwrap();
                    element.click().await.expect("Couldn't click on element");
                    println!("Tapped!");
                    steps_count += 1;
                }
                Action::InsertData { data } => {
                    println!("Inserting {} in field {:?}", data, step.selector);
                    let element = client.appium_wait().for_element(by.clone()).await.unwrap();
                    element.send_keys(&data).await.unwrap();
                }
                Action::ScrollUntilVisible => {
                    println!("Scrolling until finding: {:?}", step.selector);
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
                                println!("Founded!");
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
    let time = start.elapsed();
    println!("Test suite runned successfully");
    println!("    Actions executed: {}", steps_count);
    println!("    Total time elapsed: {:.2}", time.as_secs_f64());

    Ok(())
}
