//! Models for Android automation
//!
//! This module contains the data structures used for Android test automation.

use serde::{Deserialize, Serialize};

/// Android-specific step representation
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AndroidStep {
    /// A normal Android step with actions
    AndroidNormalStep(AndroidNormalStep),
    /// Reference to an external step file
    AndroidStepFile { step_file: String },
}

/// Types of Android normal steps
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AndroidNormalStep {
    /// Step that interacts with an element
    AndroidElementStep {
        selector: AndroidElementSelector,
        actions: Vec<AndroidAction>,
    },
    /// Step that takes a screenshot
    ScreenshotStep {
        take_screenshot: String,
    },
    /// Step that logs a message
    LogStep {
        log: String,
    },
    /// Step that pauses execution
    Pause {
        pause: u64,
    },
}

/// Actions that can be performed on Android elements
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AndroidAction {
    /// Assert that an element is visible
    AssertVisible,
    /// Tap on an element
    TapOn,
    /// Scroll until an element is visible
    ScrollUntilVisible,
    /// Insert data into an input field
    InsertData { data: String },
    /// Clear an input field
    Clear,
    /// Press the back button
    PressBack,
    /// Press a key
    PressKey { key: String },
    /// Assert that text is present in an element
    AssertText { text: String },
    /// Wait for an element to be visible
    WaitForVisible { timeout_ms: Option<u64> },
    /// Wait for an element to be invisible
    WaitForInvisible { timeout_ms: Option<u64> },
}

/// Selectors for Android elements
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AndroidElementSelector {
    /// Select by resource ID
    Id(String),
    /// Select by accessibility ID
    AccessibilityId(String),
    /// Select by XPath
    XPath(String),
    /// Select by UI Automator selector
    UiAutomator(String),
    /// Select by class name
    ClassName(String),
    /// Select by multiple criteria
    Complex {
        /// Resource ID
        id: Option<String>,
        /// Text content
        text: Option<String>,
        /// Content description
        content_desc: Option<String>,
        /// Class name
        class_name: Option<String>,
        /// XPath
        xpath: Option<String>,
        /// Index for multiple matches
        index: Option<u32>,
    },
}

/// Step file for Android
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StepFile {
    pub steps: Vec<AndroidStep>,
}
