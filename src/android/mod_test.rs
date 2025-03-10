#[cfg(test)]
mod tests {
    use crate::android::{AndroidElementSelector, get_android_element_by};
    use crate::common::CustomCapability;
    use crate::common::CustomCapabilityValue;
    use crate::android::set_custom_capabilities_android;
    use appium_client::capabilities::android::AndroidCapabilities;
    use appium_client::find::By;

    #[test]
    fn test_android_element_selector_index() {
        let selector = AndroidElementSelector::Index { index: 5 };
        let by = get_android_element_by(selector);
        
        // Check if the By instance contains the expected selector string
        // This is a simple string check since we can't directly compare By instances
        let by_debug = format!("{:?}", by);
        assert!(by_debug.contains("index(5)"));
    }

    #[test]
    fn test_android_element_selector_accessibility_id() {
        let selector = AndroidElementSelector::AccessibilityId { 
            accessibilityId: "test_id".to_string() 
        };
        let by = get_android_element_by(selector);
        
        let by_debug = format!("{:?}", by);
        assert!(by_debug.contains("test_id"));
    }

    #[test]
    fn test_android_element_selector_xpath() {
        let selector = AndroidElementSelector::Xpath { 
            xpath: "//android.widget.Button".to_string() 
        };
        let by = get_android_element_by(selector);
        
        let by_debug = format!("{:?}", by);
        assert!(by_debug.contains("//android.widget.Button"));
    }

    #[test]
    fn test_android_element_selector_text() {
        let selector = AndroidElementSelector::Text { 
            text: "Submit".to_string() 
        };
        let by = get_android_element_by(selector);
        
        let by_debug = format!("{:?}", by);
        assert!(by_debug.contains("Submit"));
    }

    #[test]
    fn test_android_element_selector_description() {
        let selector = AndroidElementSelector::Description { 
            description: "Submit button".to_string() 
        };
        let by = get_android_element_by(selector);
        
        let by_debug = format!("{:?}", by);
        assert!(by_debug.contains("Submit button"));
    }

    #[test]
    fn test_android_element_selector_hint() {
        let selector = AndroidElementSelector::Hint { 
            hint: "Enter email".to_string() 
        };
        let by = get_android_element_by(selector);
        
        let by_debug = format!("{:?}", by);
        assert!(by_debug.contains("Enter email"));
    }

    #[test]
    fn test_android_element_selector_id_with_index() {
        let selector = AndroidElementSelector::IdWithIndex { 
            id: "com.example.app:id/button".to_string(),
            index: 2
        };
        let by = get_android_element_by(selector);
        
        let by_debug = format!("{:?}", by);
        assert!(by_debug.contains("com.example.app:id/button"));
        assert!(by_debug.contains("index(2)"));
    }

    #[test]
    fn test_android_element_selector_id() {
        let selector = AndroidElementSelector::Id { 
            id: "com.example.app:id/button".to_string() 
        };
        let by = get_android_element_by(selector);
        
        let by_debug = format!("{:?}", by);
        assert!(by_debug.contains("com.example.app:id/button"));
    }

    #[test]
    fn test_android_element_selector_class_name_with_instance() {
        let selector = AndroidElementSelector::ClassName { 
            className: "android.widget.Button".to_string(),
            instance: Some(3)
        };
        let by = get_android_element_by(selector);
        
        let by_debug = format!("{:?}", by);
        assert!(by_debug.contains("android.widget.Button"));
        assert!(by_debug.contains("instance(3)"));
    }

    #[test]
    fn test_android_element_selector_class_name_without_instance() {
        let selector = AndroidElementSelector::ClassName { 
            className: "android.widget.Button".to_string(),
            instance: None
        };
        let by = get_android_element_by(selector);
        
        let by_debug = format!("{:?}", by);
        assert!(by_debug.contains("android.widget.Button"));
    }

    #[test]
    fn test_set_custom_capabilities_boolean() {
        let mut caps = AndroidCapabilities::new_uiautomator();
        let custom_caps = vec![
            CustomCapability {
                key: "noReset".to_string(),
                value: CustomCapabilityValue::BooleanValue(true),
            }
        ];
        
        set_custom_capabilities_android(&mut caps, custom_caps);
        
        // We can't directly check the capabilities, but we can verify the code doesn't panic
        // A more thorough test would require mocking or a more testable design
    }

    #[test]
    fn test_set_custom_capabilities_string() {
        let mut caps = AndroidCapabilities::new_uiautomator();
        let custom_caps = vec![
            CustomCapability {
                key: "deviceName".to_string(),
                value: CustomCapabilityValue::StringValue("Pixel 4".to_string()),
            }
        ];
        
        set_custom_capabilities_android(&mut caps, custom_caps);
        
        // We can't directly check the capabilities, but we can verify the code doesn't panic
    }

    #[test]
    fn test_set_custom_capabilities_number() {
        let mut caps = AndroidCapabilities::new_uiautomator();
        let custom_caps = vec![
            CustomCapability {
                key: "newCommandTimeout".to_string(),
                value: CustomCapabilityValue::NumberValue(60.0),
            }
        ];
        
        set_custom_capabilities_android(&mut caps, custom_caps);
        
        // We can't directly check the capabilities, but we can verify the code doesn't panic
    }

    #[test]
    fn test_set_custom_capabilities_null() {
        let mut caps = AndroidCapabilities::new_uiautomator();
        let custom_caps = vec![
            CustomCapability {
                key: "someNullCapability".to_string(),
                value: CustomCapabilityValue::NullValue,
            }
        ];
        
        set_custom_capabilities_android(&mut caps, custom_caps);
        
        // We can't directly check the capabilities, but we can verify the code doesn't panic
    }
}
