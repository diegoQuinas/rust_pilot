use colored::Colorize;
use std::fmt::Display;

pub struct Logger;

impl Logger {
    // Info messages - blue color with ℹ️ icon
    pub fn info<T: Display>(message: T) {
        println!("ℹ️  {}", message.to_string().blue());
    }

    pub fn info_with_indent<T: Display>(message: T, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{}ℹ️  {}", indent, message.to_string().blue());
    }

    // Success messages - green color with ✅ icon
    pub fn success<T: Display>(message: T) {
        println!("✅ {}", message.to_string().green());
    }

    pub fn success_with_indent<T: Display>(message: T, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{}✅ {}", indent, message.to_string().green());
    }

    // Error messages - red color with ❌ icon
    pub fn error<T: Display>(message: T) {
        eprintln!("❌ {}", message.to_string().red());
    }

    pub fn error_with_indent<T: Display>(message: T, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        eprintln!("{}❌ {}", indent, message.to_string().red());
    }

    // Warning messages - yellow color with ⚠️ icon
    pub fn warning<T: Display>(message: T) {
        println!("⚠️  {}", message.to_string().yellow());
    }

    pub fn warning_with_indent<T: Display>(message: T, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{}⚠️  {}", indent, message.to_string().yellow());
    }

    // Step messages - cyan color with 👉 icon
    pub fn step<T: Display>(message: T) {
        println!("⏳ {}", message.to_string().cyan());
    }

    pub fn step_with_indent<T: Display>(message: T, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{}⏳ {}", indent, message.to_string().cyan());
    }
}
