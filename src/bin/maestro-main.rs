use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    time::Instant,
};

use chrono::Local;
use colored::Colorize;
use maestro_rs::{
    android::*,
    shared::*,
    tags::{error_tag, info_tag},
};

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Test file path and caps path needed as argument");
        return Ok(());
    }
    if args.len() < 3 {
        println!("Caps path needed as argument");
        return Ok(());
    }

    print_logo();

    let file_path = &args[1];
    let caps_path = &args[2];

    println!("{} Test file path: {}", info_tag(), file_path.blue());
    println!("{} Caps file path: {}", info_tag(), caps_path.blue());

    let (_, steps) = parse_test_file(file_path);

    let base_path = Path::new(file_path)
        .parent()
        .expect("Failed to determine base path");

    let mermaid_base_id = format!("idRoot0({})", base_path.display());
    let (flattened_steps, mermaid_steps) = flatten_steps(steps, base_path, mermaid_base_id).await;

    let mut caps_file = File::open(caps_path)?;
    let mut caps_contents = String::new();
    caps_file.read_to_string(&mut caps_contents)?;

    let capabilities_file: CapsFile = serde_yaml::from_str(&caps_contents)?;

    let start = Instant::now();
    let (steps_count, report) = match capabilities_file.platform {
        Platform::Ios => {
            /*launch_ios_main(capabilities, steps).await.unwrap()*/
            (0, "### IOS NOT IMPLEMENTED".to_string())
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
    };

    let time = start.elapsed();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let full_report = format!(
        "# Test suite report\n\n![LOGO](./assets/logo.webp)\n\nTest file: {}\n\nPlatform: Android\n\n🕒 Date and time {}\n\n✅ Steps executed {} successfully\n\n{}",
        file_path, now, steps_count, report
    );
    let report_name = format!("REPORT_{}.md", Local::now().format("%Y-%m%d_%H-%M-%S"));
    if let Ok(mut report_file) = fs::File::create(&report_name) {
        let _ = report_file.write_all(full_report.as_bytes());
    } else {
        eprintln!("{} Error creating report file", error_tag());
    }
    println!("\n\nTest suite runned successfully");
    println!("    Report file: {}", report_name);
    println!("    Actions executed: {}", steps_count);
    println!("    Total time elapsed: {:.2} seconds", time.as_secs_f64());
    Ok(())
}
