use appium_client::{
    capabilities::{
        android::AndroidCapabilities, AppCapable, AppiumCapability, UiAutomator2AppCompatible,
    },
    find::{AppiumFind, By},
    wait::AppiumWait,
    ClientBuilder,
};
use fantoccini::{
    actions::{InputSource, PointerAction, TouchActions, MOUSE_BUTTON_LEFT},
    Client,
};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read};
use tokio::time::{timeout, Duration};

#[derive(Debug, Serialize, Deserialize)]
struct TestFile {
    steps: Vec<Step>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Step {
    AssertVisible { assert_visible: ElementProps },
    TapOn { tap_on: ElementProps },
    ScrollUntilVisible { scroll_until_visible: ElementProps },
}

#[derive(Debug, Serialize, Deserialize)]
struct ElementProps {
    text: Option<String>,
    xpath: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the YAML file
    let mut file = File::open("test.yaml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Deserialize the YAML into our TestFile struct
    let test_file: TestFile = serde_yaml::from_str(&contents)?;

    println!("{:?}", test_file.steps);
    // Configure the Appium driver
    let mut caps = AndroidCapabilities::new_uiautomator();
    caps.app("./app/wikipedia.apk");
    caps.platform_version("13");
    //caps.app_wait_package("com.cencosud.parisapp.welcome.WelcomeActivity");
    caps.set_bool("appium:autoGrantPermissions", true);
    caps.app_wait_package("org.wikipedia");
    caps.app_wait_activity("org.wikipedia.onboarding.InitialOnboardingActivity");
    caps.full_reset(true);
    //caps.app_package(&test_file.app_id);

    println!("Launching app");
    let client = ClientBuilder::native(caps)
        .connect("http://localhost:4723/")
        .await?;

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

    async fn assert_visible_element(client: Client, text: &str) -> bool {
        let element = client
            .appium_wait()
            .for_element(By::uiautomator(&format!(
                "new UiSelector().text(\"{}\");",
                text
            )))
            .await
            .unwrap();
        element.is_displayed().await.unwrap()
    }

    for step in test_file.steps {
        match step {
            Step::AssertVisible { assert_visible } => {
                let text = assert_visible.text.unwrap();
                //let xpath = element.xpath.unwrap()

                println!("Asserting visible: {}", text);
                assert!(assert_visible_element(client.clone(), &text).await);
            }
            Step::TapOn { tap_on } => {
                let text = tap_on.text.unwrap();
                println!("Tapping on: {}", text);
                let element = client
                    .appium_wait()
                    .for_element(By::uiautomator(&format!(
                        "new UiSelector().text(\"{}\");",
                        text
                    )))
                    .await?;
                element.click().await?;
            }
            Step::ScrollUntilVisible {
                scroll_until_visible,
            } => {
                let text = scroll_until_visible.text.unwrap();
                println!("Scrolling until visible: {}", text);
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
                        if let Ok(_) = client
                            .clone()
                            .find_by(By::uiautomator(&format!(
                                "new UiSelector().text(\"{}\");",
                                text
                            )))
                            .await
                        {
                            println!("Founded!");
                            visible = true;
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

    Ok(())
}
