#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::time::Duration;
    use crate::reporting::TestReport;

    #[test]
    fn test_report_directory_creation() {
        // Setup: Ensure reports directory doesn't exist
        let reports_dir = Path::new("test_reports");
        if reports_dir.exists() {
            fs::remove_dir_all(reports_dir).expect("Failed to remove test_reports directory");
        }

        // Create a test report
        let report = TestReport {
            test_file: "test_file.yml".to_string(),
            platform: "Test".to_string(),
            steps_executed: 5,
            execution_time: Duration::from_secs(10),
            details: "Test details".to_string(),
        };

        // Save the report to a custom directory
        let save_result = report.save_to_dir("test_reports");
        assert!(save_result.is_ok(), "Failed to save report: {:?}", save_result.err());

        // Verify directory was created
        assert!(reports_dir.exists(), "Reports directory was not created");

        // Verify file was created
        let report_path = save_result.unwrap();
        assert!(Path::new(&report_path).exists(), "Report file was not created");

        // Cleanup
        fs::remove_dir_all(reports_dir).expect("Failed to clean up test_reports directory");
    }

    #[test]
    fn test_report_content() {
        // Create a test report with known values
        let report = TestReport {
            test_file: "test_file.yml".to_string(),
            platform: "Test Platform".to_string(),
            steps_executed: 10,
            execution_time: Duration::from_secs(30),
            details: "Test execution details".to_string(),
        };

        // Generate markdown content
        let markdown = report.generate_markdown();

        // Verify content contains expected information
        assert!(markdown.contains("test_file.yml"), "Report doesn't contain test file name");
        assert!(markdown.contains("Test Platform"), "Report doesn't contain platform name");
        assert!(markdown.contains("Steps executed: 10"), "Report doesn't contain correct step count");
        assert!(markdown.contains("30.00 seconds"), "Report doesn't contain correct execution time");
        assert!(markdown.contains("Test execution details"), "Report doesn't contain details");
    }
    
    #[test]
    fn test_report_new() {
        // Test the constructor method
        let report = TestReport::new("test_file.yml".to_string(), "Android".to_string());
        
        // Verify initial values
        assert_eq!(report.test_file, "test_file.yml");
        assert_eq!(report.platform, "Android");
        assert_eq!(report.steps_executed, 0);
        assert_eq!(report.execution_time, Duration::from_secs(0));
        assert_eq!(report.details, "");
    }
    
    #[test]
    fn test_report_default_save() {
        // Setup: Ensure reports directory doesn't exist
        let reports_dir = Path::new("reports");
        if reports_dir.exists() {
            fs::remove_dir_all(reports_dir).expect("Failed to remove reports directory");
        }
        
        // Create a test report
        let report = TestReport {
            test_file: "test_file.yml".to_string(),
            platform: "Test".to_string(),
            steps_executed: 5,
            execution_time: Duration::from_secs(10),
            details: "Test details".to_string(),
        };
        
        // Save the report to the default directory
        let save_result = report.save();
        assert!(save_result.is_ok(), "Failed to save report to default directory: {:?}", save_result.err());
        
        // Verify directory was created
        assert!(reports_dir.exists(), "Default reports directory was not created");
        
        // Verify file was created
        let report_path = save_result.unwrap();
        assert!(Path::new(&report_path).exists(), "Report file was not created in default directory");
        
        // Cleanup
        fs::remove_dir_all(reports_dir).expect("Failed to clean up reports directory");
    }
    
    #[test]
    fn test_report_with_existing_directory() {
        // Setup: Create the directory first
        let reports_dir = Path::new("existing_reports");
        if !reports_dir.exists() {
            fs::create_dir(reports_dir).expect("Failed to create existing_reports directory");
        }
        
        // Create a test report
        let report = TestReport {
            test_file: "test_file.yml".to_string(),
            platform: "Test".to_string(),
            steps_executed: 5,
            execution_time: Duration::from_secs(10),
            details: "Test details".to_string(),
        };
        
        // Save the report to the existing directory
        let save_result = report.save_to_dir("existing_reports");
        assert!(save_result.is_ok(), "Failed to save report to existing directory: {:?}", save_result.err());
        
        // Verify file was created
        let report_path = save_result.unwrap();
        assert!(Path::new(&report_path).exists(), "Report file was not created in existing directory");
        
        // Cleanup
        fs::remove_dir_all(reports_dir).expect("Failed to clean up existing_reports directory");
    }
}
