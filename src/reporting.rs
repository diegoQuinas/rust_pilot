use chrono::Local;
use std::{fs::{self, File}, io::Write, path::Path, time::Duration};

pub struct TestReport {
    pub test_file: String,
    pub platform: String,
    pub steps_executed: usize,
    pub execution_time: Duration,
    pub details: String,
}

impl TestReport {
    pub fn new(test_file: String, platform: String) -> Self {
        TestReport {
            test_file,
            platform,
            steps_executed: 0,
            execution_time: Duration::from_secs(0),
            details: String::new(),
        }
    }

    pub fn generate_markdown(&self) -> String {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S");
        format!(
            "# Test suite report\n\n\
            ![LOGO](./assets/logo.webp)\n\n\
            Test file: {}\n\n\
            Platform: {}\n\n\
            🕒 Date and time: {}\n\n\
            ✅ Steps executed: {} successfully\n\n\
            ⏱️ Total execution time: {:.2} seconds\n\n\
            ## Test Details\n\n\
            {}\n",
            self.test_file,
            self.platform,
            now,
            self.steps_executed,
            self.execution_time.as_secs_f64(),
            self.details
        )
    }

    pub fn save(&self) -> Result<String, Box<dyn std::error::Error>> {
        self.save_to_dir("reports")
    }

    pub fn save_to_dir(&self, dir_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Ensure directory exists
        let dir_path = Path::new(dir_name);
        if !dir_path.exists() {
            fs::create_dir(dir_path)?;
        }
        
        let report_name = format!(
            "{}/REPORT_{}.md",
            dir_name,
            Local::now().format("%Y%m%d_%H-%M-%S")
        );
        
        let mut report_file = File::create(&report_name)?;
        report_file.write_all(self.generate_markdown().as_bytes())?;
        
        Ok(report_name)
    }
}
