use std::{process, time::Duration};

use appium_client::{capabilities::android::AndroidCapabilities, wait::AppiumWait, Client};

use crate::{
    android::{get_android_element_by, AndroidElementSelector},
    shared::start_spinner,
    tags::{error_tag, info_tag},
};

use super::{stop_spinner, Step, TapOn};

pub async fn execute_android_steps(client: &Client<AndroidCapabilities>, steps: Vec<Step>) -> u32 {
    let mut steps_count = 0;
    for step in steps {
        match step {
            Step::AssertVisible { assertVisible } => {
                let selector = AndroidElementSelector::Text {
                    text: assertVisible,
                };
                let by = get_android_element_by(selector.clone());
                let mut sp = start_spinner(format!("Asserting visible: {:?}", selector));
                let element = client.appium_wait().for_element(by.clone()).await.unwrap();
                let is_visible = element.is_displayed().await.unwrap();
                assert!(is_visible);
                steps_count += 1;
                stop_spinner(&mut sp)
            }
            Step::AssertNotVisible { assertNotVisible } => {
                let selector = AndroidElementSelector::Text {
                    text: assertNotVisible.clone(),
                };
                let by = get_android_element_by(selector.clone());
                let mut sp = start_spinner(format!("Asserting not visible: {:?}", selector));
                if let Ok(element) = client
                    .appium_wait()
                    .at_most(Duration::from_millis(1000))
                    .for_element(by.clone())
                    .await
                {
                    let is_visible = element.is_displayed().await.unwrap();
                    assert!(!is_visible);
                    sp.stop_with_symbol(&format!(
                        "{} Element {} visible",
                        error_tag(),
                        assertNotVisible.clone()
                    ));
                    process::exit(1);
                };
                stop_spinner(&mut sp);
            }
            Step::TapOn { tapOn } => {
                let selector = match tapOn {
                    TapOn::TapOnText(text) => AndroidElementSelector::Text { text },
                    TapOn::TapOnOption(tap_on_options) => {
                        if let Some(class_name) = tap_on_options.optional {
                            panic!("NOT DEVELOPED OPTIONAL");
                        }
                        if let Some(text) = tap_on_options.text {
                            AndroidElementSelector::Text { text }
                        } else if let Some(id) = tap_on_options.id {
                            if let Some(index) = tap_on_options.index {
                                AndroidElementSelector::IdWithIndex { id, index }
                            } else {
                                AndroidElementSelector::Id { id }
                            }
                        } else {
                            panic!("NOT DEVELOPED TAP ON OPTION");
                        }
                    }
                };
                let by = get_android_element_by(selector.clone());
                let mut sp = start_spinner(format!("Tapping on: {:?}", selector));
                let element = client.appium_wait().for_element(by.clone()).await.unwrap();
                element.click().await.expect("Couldn't click on element");
                steps_count += 1;
                stop_spinner(&mut sp)
            }
            Step::InputText { inputText } => {
                let mut sp = start_spinner(format!("Inserting {} ", inputText,));
                client
                    .execute(
                        "mobile: type",
                        vec![serde_json::json!({ "text": &inputText})],
                    )
                    .await
                    .unwrap();
                stop_spinner(&mut sp);
            }
            Step::RunScript { runScript } => {
                let mut sp = start_spinner(format!("Running script: {}", runScript));
                stop_spinner(&mut sp)
            }
            other => {
                println!("{} Step {:?} not developed", info_tag(), other)
            }
        }
    }
    steps_count
}

/*AndroidNormalStep::AndroidElementStep { selector, actions } => {
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
*/
