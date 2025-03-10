use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Read};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub platform_name: String,
    pub capabilities: HashMap<String, Value>,
}

impl Config {
    pub fn from_file(caps_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut caps_file = File::open(caps_path)?;
        let mut caps_contents = String::new();
        caps_file.read_to_string(&mut caps_contents)?;

        let capabilities: HashMap<String, Value> = serde_json::from_str(&caps_contents)?;
        
        let platform_name = capabilities
            .get("platformName")
            .and_then(|v| v.as_str())
            .ok_or("Missing or invalid platformName in capabilities")?
            .to_string();

        Ok(Config {
            platform_name,
            capabilities,
        })
    }
}
