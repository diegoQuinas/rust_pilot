#[cfg(test)]
mod tests {
    use crate::common::utils::{get_current_indent_level, set_current_indent_level, PlainLogger};

    #[test]
    fn test_indent_level() {
        // Set the indent level
        set_current_indent_level(3);
        
        // Check if the indent level is correctly set
        assert_eq!(get_current_indent_level(), 3);
        
        // Change the indent level
        set_current_indent_level(5);
        
        // Check if the indent level is correctly updated
        assert_eq!(get_current_indent_level(), 5);
        
        // Reset the indent level for other tests
        set_current_indent_level(0);
    }

    #[test]
    fn test_plain_logger_creation() {
        let message = "Test message".to_string();
        let logger = PlainLogger {
            message: message.clone(),
            indent_level: 0,
        };
        
        assert_eq!(logger.message, message);
        assert_eq!(logger.indent_level, 0);
    }
}
