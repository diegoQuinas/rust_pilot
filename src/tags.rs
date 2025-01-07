use colored::Colorize;

pub fn ok_tag() -> String {
    "[OK]".green().to_string()
}
pub fn error_tag() -> String {
    "[ERR]".red().to_string()
}

pub fn log_tag() -> String {
    "[LOG]".on_white().black().bold().to_string()
}
pub fn info_tag() -> String {
    "[INFO]".blue().to_string()
}
pub fn warning_tag() -> String {
    "[WARN]".yellow().to_string()
}
