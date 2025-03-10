use std::{
    collections::HashMap,
    env,
    fs::File,
    io::Read,
    path::Path,
    process,
    time::Instant,
};

use chrono::Local;
use colored::Colorize;
use rust_pilot::{
    android::*,
    common::{tags::*, *},

    logger::Logger,
    reporting::TestReport,
};
use serde_json::Value;

const LOGO: &str = r#"
  _____                  _     _____    _   _           _   
 |  __ \                | |   |  __ \  (_) | |         | |  
 | |__) |  _   _   ___  | |_  | |__) |  _  | |   ___   | |_ 
 |  _  /  | | | | / __| | __| |  ___/  | | | |  / _ \  | __|
 | | \ \  | |_| | \__ \ | |_  | |      | | | | | (_) | | |_ 
 |_|  \_\  \__,_| |___/  \__| |_|      |_| |_|  \___/   \__|
"#;

const USAGE: &str = "Usage: rp <caps_file> <test_file>";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    display_startup_info();

    let (caps_path, test_file_path) = parse_command_line_args()?;

    Logger::info(format!("Caps file path: {}", caps_path));
    Logger::info(format!("Test file path: {}", test_file_path));

    let (_header, steps) = parse_test_file(&test_file_path);

    let base_path = Path::new(&test_file_path)
        .parent()
        .ok_or("Failed to determine base path")?;

    let (flattened_steps, _) = flatten_steps(
        steps,
        base_path,
        format!("idRoot0({})", base_path.display()),
    )
    .await;

    let mut caps_file = File::open(caps_path)?;
    let mut caps_contents = String::new();
    caps_file.read_to_string(&mut caps_contents)?;

    let capabilities_file: HashMap<String, Value> = serde_json::from_str(&caps_contents)?;

    let start = Instant::now();
    let (steps_count, report) = match capabilities_file.get("platformName") {
        Some(Value::String(platform)) => match platform.as_str() {
            "android" => launch_android_main(&capabilities_file, flattened_steps)
                .await
                .unwrap_or_else(|err| {
                    eprintln!("{} Error launching Android test: {}", error_tag(), err);
                    (
                        0,
                        format!("### ERROR LAUNCHING ANDROID TEST\n```{}```", err),
                    )
                }),
            "ios" => {
                todo!("### IOS NOT IMPLEMENTED")
            }
            _ => {
                eprintln!("{} Invalid platform", error_tag());
                process::exit(1);
            }
        },
        None => {
            eprintln!("{} Missing platform key in caps file", error_tag());
            process::exit(1);
        }
        Some(_) => {
            eprintln!("{} Invalid platform key in caps file", error_tag());
            process::exit(1);
        }
    };

    let time = start.elapsed();
    let _now = Local::now().format("%Y-%m-%d %H:%M:%S");

    let mut test_report = TestReport::new(test_file_path.clone(), "Android".to_string());
    test_report.steps_executed = steps_count;
    test_report.execution_time = time;
    test_report.details = report;

    let report_name = test_report.save().unwrap_or_else(|e| {
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

fn display_startup_info() {
    // Reset indentation level for startup info
    rust_pilot::common::set_current_indent_level(0);
    println!();
    println!("{}", LOGO.yellow());
    Logger::info(format!("rust_pilot version: {}", env!("CARGO_PKG_VERSION")));
    println!();
}

fn parse_command_line_args() -> Result<(String, String), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        Logger::error("Missing arguments");
        Logger::error(USAGE);
        process::exit(1);
    }

    Ok((args[1].clone(), args[2].clone()))
}
