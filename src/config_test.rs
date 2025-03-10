#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use serde_json::json;
    use serde_json::Value;
    
    use crate::config::Config;

    // Helper function to create a temporary config file
    fn create_test_config_file(file_path: &str, platform_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut capabilities = HashMap::new();
        capabilities.insert("platformName".to_string(), Value::String(platform_name.to_string()));
        capabilities.insert("deviceName".to_string(), Value::String("Test Device".to_string()));
        capabilities.insert("automationName".to_string(), Value::String("UiAutomator2".to_string()));
        capabilities.insert("appium:app".to_string(), Value::String("/path/to/app.apk".to_string()));
        
        let json_content = serde_json::to_string_pretty(&capabilities)?;
        
        let mut file = File::create(file_path)?;
        file.write_all(json_content.as_bytes())?;
        
        Ok(())
    }

    #[test]
    fn test_config_from_file() {
        // Setup: Create a temporary config file
        let config_path = "test_config.json";
        create_test_config_file(config_path, "Android").expect("Failed to create test config file");
        
        // Test loading the config from file
        let config_result = Config::from_file(config_path);
        assert!(config_result.is_ok(), "Failed to load config: {:?}", config_result.err());
        
        let config = config_result.unwrap();
        
        // Verify the config was loaded correctly
        assert_eq!(config.platform_name, "Android");
        assert_eq!(config.capabilities.len(), 4);
        assert_eq!(
            config.capabilities.get("deviceName").and_then(|v| v.as_str()),
            Some("Test Device")
        );
        
        // Cleanup
        fs::remove_file(config_path).expect("Failed to remove test config file");
    }
    
    #[test]
    fn test_config_missing_platform_name() {
        // Setup: Create a config file without platform name
        let config_path = "test_config_missing.json";
        
        let mut capabilities = HashMap::new();
        capabilities.insert("deviceName".to_string(), Value::String("Test Device".to_string()));
        
        let json_content = serde_json::to_string_pretty(&capabilities).unwrap();
        let mut file = File::create(config_path).unwrap();
        file.write_all(json_content.as_bytes()).unwrap();
        
        // Test loading the config from file
        let config_result = Config::from_file(config_path);
        
        // Verify the error is returned for missing platform name
        assert!(config_result.is_err());
        
        // Cleanup
        fs::remove_file(config_path).expect("Failed to remove test config file");
    }
    
    #[test]
    fn test_config_invalid_json() {
        // Setup: Create an invalid JSON file
        let config_path = "test_config_invalid.json";
        
        let invalid_json = r#"{ "platformName": "Android", "deviceName": "Test Device" "#; // Missing closing brace
        let mut file = File::create(config_path).unwrap();
        file.write_all(invalid_json.as_bytes()).unwrap();
        
        // Test loading the config from file
        let config_result = Config::from_file(config_path);
        
        // Verify the error is returned for invalid JSON
        assert!(config_result.is_err());
        
        // Cleanup
        fs::remove_file(config_path).expect("Failed to remove test config file");
    }
    
    #[test]
    fn test_config_file_not_found() {
        // Test loading a non-existent config file
        let config_result = Config::from_file("non_existent_config.json");
        
        // Verify the error is returned for file not found
        assert!(config_result.is_err());
    }
}
