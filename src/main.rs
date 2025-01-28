use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::Path,
    process,
    time::Instant,
};

use chrono::Local;
use colored::Colorize;
use rust_pilot::{
    android::*,
    common::{
        tags::{error_tag, info_tag},
        *,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    validate_args(&args)?;

    print_spaces();
    print_logo();
    print_version();
    print_spaces();

    let caps_path = &args[1];
    let file_path = &args[2];

    println!("{} Caps file path: {}", info_tag(), caps_path.blue());
    println!("{} Test file path: {}", info_tag(), file_path.blue());

    let (_header, steps) = parse_test_file(&file_path);

    let base_path = Path::new(file_path)
        .parent()
        .expect("Failed to determine base path");

    let mermaid_base_id = format!("idRoot0({})", base_path.display());
    let (flattened_steps, _mermaid_steps) = flatten_steps(steps, base_path, mermaid_base_id).await;

    let mut caps_file = File::open(caps_path)?;
    let mut caps_contents = String::new();
    caps_file.read_to_string(&mut caps_contents)?;

    let capabilities_file: CapsFile = serde_yaml::from_str(&caps_contents)?;

    let start = Instant::now();
    let (steps_count, report) = match capabilities_file.platform {
        Platform::Ios => {
            todo!("### IOS NOT IMPLEMENTED")
        }
        Platform::Android => launch_android_main(capabilities_file, flattened_steps)
            .await
            .unwrap_or_else(|err| {
                eprintln!("{} Error launching Android test: {}", error_tag(), err);
                (
                    0,
                    format!("### ERROR LAUNCHING ANDROID TEST\n```{}```", err),
                )
            }),
        Platform::Flutter => {
            todo!("### FLUTTER NOT IMPLEMENTED")
        }
    };

    let time = start.elapsed();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let full_report = format!(
        "# Test suite report\n\n![LOGO](./assets/logo.webp)\n\nTest file: {}\n\nPlatform: Android\n\n🕒 Date and time {}\n\n✅ Steps executed {} successfully\n\n{}",
        file_path, now, steps_count, report
    );
    let report_name = format!(
        "reports/REPORT_{}.md",
        Local::now().format("%Y-%m%d_%H-%M-%S")
    );

    write_report(&report_name, &full_report).unwrap_or_else(|e| {
        eprintln!(
            "{} Error writing report file: {}",
            error_tag(),
            e.to_string().red()
        );
        process::exit(1);
    });

    println!("\n\n{}", "Test suite runned successfully".green());
    println!("    Report file: {}", report_name);
    println!("    Actions executed: {}", steps_count);
    println!("    Total time elapsed: {:.2} seconds", time.as_secs_f64());
    Ok(())
}

fn validate_args(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 3 {
        eprintln!("{} Missing arguments", error_tag());
        eprintln!("{} Usage: rp <caps_file> <test_file>", error_tag());
        process::exit(1);
    }
    Ok(())
}

fn print_spaces() {
    println!();
}
fn print_logo() {
    let logo = r#"
  _____                  _     _____    _   _           _   
 |  __ \                | |   |  __ \  (_) | |         | |  
 | |__) |  _   _   ___  | |_  | |__) |  _  | |   ___   | |_ 
 |  _  /  | | | | / __| | __| |  ___/  | | | |  / _ \  | __|
 | | \ \  | |_| | \__ \ | |_  | |      | | | | | (_) | | |_ 
 |_|  \_\  \__,_| |___/  \__| |_|      |_| |_|  \___/   \__|
                                                         "#;
    println!("{}", logo.yellow());
}

fn print_version() {
    let version = env!("CARGO_PKG_VERSION");
    println!("rust_pilot version: {}", version);
}

fn write_report(report_name: &str, full_report: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut report_file = File::create(&report_name)?;
    report_file.write_all(full_report.as_bytes())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_args() {
        let args = vec!["rp".to_string()];
        assert!(validate_args(&args).is_err());
    }
}
