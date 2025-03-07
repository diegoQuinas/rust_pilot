use std::{process, time::Duration};

use appium_client::{capabilities::android::AndroidCapabilities, wait::AppiumWait, Client};
use fantoccini::actions::{InputSource, PointerAction, TouchActions, MOUSE_BUTTON_LEFT};

use crate::{
    android::{get_android_element_by, AndroidElementSelector},
    common::tags::{error_tag, info_tag, ok_tag, valid_report_tag},
    common::{error_take_screenshot, start_spinner},
};

use super::{Step, TapOn};

pub async fn execute_android_steps(
    client: &Client<AndroidCapabilities>,
    steps: Vec<Step>,
) -> (u32, String) {
    let mut report =
        "### Android Steps\n| Description | State | Observation | \n |----|----|----|\n"
            .to_string();
    let steps_count = steps.len() as u32;
    for step in steps {
        match step {
            Step::Swipe { swipe } => {
                let device_size = client.get_window_size().await.unwrap();
                let start = swipe.start.to_f64();
                let end = swipe.end.to_f64();
                let x_y_from: (i64, i64) = (
                    (device_size.0 as f64 * (start.0) / 100.0).round() as i64,
                    (device_size.1 as f64 * (start.1) / 100.0).round() as i64,
                );
                let x_y_end: (i64, i64) = (
                    (device_size.0 as f64 * (end.0) / 100.0).round() as i64,
                    (device_size.1 as f64 * (end.1) / 100.0).round() as i64,
                );
                let swipe_options = swipe;
                let mut sp = start_spinner(format!("Swiping: {:?}", swipe_options));
                let swipe_down = TouchActions::new("finger".to_string())
                    .then(PointerAction::MoveTo {
                        duration: Some(Duration::from_millis(0)),
                        x: x_y_from.0,
                        y: x_y_from.1,
                    })
                    .then(PointerAction::Down {
                        button: MOUSE_BUTTON_LEFT,
                    })
                    .then(PointerAction::MoveTo {
                        duration: Some(Duration::from_millis(500)),
                        x: x_y_end.0,
                        y: x_y_end.1,
                    });

                match client.perform_actions(swipe_down.clone()).await {
                    Ok(_) => {}
                    Err(err) => {
                        sp.stop_with_symbol(&format!("{} Error swiping: {:?}", error_tag(), err));
                        error_take_screenshot(&client).await;
                        process::exit(1);
                    }
                }

                sp.stop_with_symbol(&format!("{} Swiped {:?}", ok_tag(), swipe_options));
            }
            Step::AssertVisible { assertVisible } => {
                let selector = AndroidElementSelector::Text {
                    text: assertVisible.clone(),
                };
                let by = get_android_element_by(selector.clone());
                let mut sp = start_spinner(format!("Asserting visible: {:?}", selector));
                let element = match client.appium_wait().for_element(by.clone()).await {
                    Ok(element) => element,
                    Err(err) => {
                        sp.stop_with_symbol(&format!(
                            "{} Error finding element: {:?}",
                            error_tag(),
                            err
                        ));
                        error_take_screenshot(&client).await;
                        process::exit(1);
                    }
                };
                let is_visible = element.is_displayed().await.unwrap();
                assert!(is_visible);
                sp.stop_with_symbol(&format!(
                    "{} Element {} visible",
                    ok_tag(),
                    assertVisible.clone()
                ));
                report.push_str(&format!(
                    "| Element {} visible | {} |  |\n",
                    assertVisible.clone(),
                    valid_report_tag()
                ));
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
                    error_take_screenshot(&client).await;
                    process::exit(1);
                };
                sp.stop_with_symbol(&format!(
                    "{} Element {} not visible",
                    ok_tag(),
                    assertNotVisible.clone()
                ));
                report.push_str(&format!(
                    "| Element {} not visible | {} |  |\n",
                    assertNotVisible.clone(),
                    valid_report_tag()
                ));
            }
            Step::TapOn { tapOn } => match tapOn {
                TapOn::TapOnTextOrDescription(string) => {
                    let selector_text = AndroidElementSelector::Text {
                        text: string.clone(),
                    };
                    let by = get_android_element_by(selector_text);
                    let mut sp = start_spinner(format!("Tapping on text: {:?}", string));
                    let element = client
                        .appium_wait()
                        .at_most(Duration::from_millis(1000))
                        .for_element(by.clone())
                        .await;
                    if let Ok(element) = element {
                        element.click().await.expect("Couldn't click on element");
                        sp.stop_with_symbol(&format!("{} Tapped on text: {}", ok_tag(), string));
                        report.push_str(&format!(
                            "| Tapped on text: {} | {} |  |\n",
                            string,
                            valid_report_tag()
                        ));
                    } else {
                        sp.stop_with_symbol(&format!(
                            "{} Can't find text: {}, tying with description",
                            info_tag(),
                            string
                        ));
                        let mut spinner =
                            start_spinner(format!("Tapping on description: {}", string));
                        let selector_description = AndroidElementSelector::Description {
                            description: string.clone(),
                        };
                        let by_description = get_android_element_by(selector_description);
                        let element_description = client
                            .appium_wait()
                            .for_element(by_description.clone())
                            .await;
                        if let Ok(element_description) = element_description {
                            element_description
                                .click()
                                .await
                                .expect("Couldn't click on element");
                            spinner.stop_with_symbol(&format!(
                                "{} Tapped on description: {}",
                                ok_tag(),
                                string
                            ));
                            report.push_str(&format!(
                                "| Tapped on description: {} | {} | |\n",
                                string,
                                valid_report_tag()
                            ));
                        } else {
                            spinner.stop_with_symbol(&format!(
                                "{} Can't find description: {}",
                                error_tag(),
                                string
                            ));
                            error_take_screenshot(&client).await;
                            process::exit(1);
                        }
                    }
                }
                TapOn::TapOnOption(tap_on_options) => {
                    let selector = {
                        #[allow(unused_variables)]
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
                        } else if let Some(index) = tap_on_options.index {
                            AndroidElementSelector::Index { index}
                        } else if let Some(description) = tap_on_options.description {
                            AndroidElementSelector::Description { description }
                        } else if let Some(class_name) = tap_on_options.className {
                            if let Some(instance) = tap_on_options.instance {
                                AndroidElementSelector::ClassName {
                                    className: class_name,
                                    instance: Some(instance),
                                }
                            } else {
                                AndroidElementSelector::ClassName {
                                    className: class_name,
                                    instance: None,
                                }
                            }
                        } else if let Some(hint) = tap_on_options.hint {
                            AndroidElementSelector::Hint { hint} 
                        } else {
                            eprintln!("{} NOT DEVELOPED TAP ON OPTION", error_tag());
                            process::exit(1)
                        }
                    };
                    let by = get_android_element_by(selector.clone());
                    let mut sp = start_spinner(format!("Tapping on: {:?}", selector));
                    let element = client.appium_wait().for_element(by.clone()).await.unwrap();
                    element.click().await.expect("Couldn't click on element");
                    sp.stop_with_symbol(&format!("{} Tapped on: {:?}", ok_tag(), selector.clone()));
                    report.push_str(&format!(
                        "| Tapped on: {:?} | {} |  |\n",
                        selector.clone(),
                        valid_report_tag()
                    ));
                }
            },
            Step::InputText { inputText } => {
                let mut sp = start_spinner(format!("Inserting {} ", inputText,));
                client
                    .execute(
                        "mobile: type",
                        vec![serde_json::json!({ "text": &inputText})],
                    )
                    .await
                    .unwrap();
                sp.stop_with_symbol(&format!("{} Inserted {}", ok_tag(), inputText));
                report.push_str(&format!(
                    "| Inserted {} | {} |  |\n",
                    inputText,
                    valid_report_tag()
                ));
            }
            Step::RunScript { runScript } => {
                let mut sp = start_spinner(format!("Running script: {}", runScript));
                sp.stop_with_symbol(&format!("{} Ran script: {}", ok_tag(), runScript));
            }
            other => {
                println!("{} Step {:?} not developed", info_tag(), other)
            }
        }
    }
    (steps_count, report)
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
