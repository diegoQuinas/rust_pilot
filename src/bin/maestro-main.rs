use std::{env, fs::File, io::Read, path::Path, time::Instant};

use colored::Colorize;
use maestro_rs::{
    android::*,
    shared::*,
    tags::{error_tag, info_tag},
};

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

    let file_path = &args[1];
    let caps_path = &args[2];

    println!("{} Test file path: {}", info_tag(), file_path.blue());
    println!("{} Caps file path: {}", info_tag(), caps_path.blue());

    let (_, steps) = parse_test_file(file_path);

    let base_path = Path::new(file_path)
        .parent()
        .expect("Failed to determine base path");

    let flattened_steps = flatten_steps(steps, base_path).await;

    let mut caps_file = File::open(caps_path)?;
    let mut caps_contents = String::new();
    caps_file.read_to_string(&mut caps_contents)?;

    let capabilities_file: CapsFile = serde_yaml::from_str(&caps_contents)?;

    let start = Instant::now();
    let steps_count = match capabilities_file.platform {
        Platform::Ios => {
            /*launch_ios_main(capabilities, steps).await.unwrap()*/
            0
        }
        Platform::Android => launch_android_main(capabilities_file, flattened_steps)
            .await
            .unwrap_or_else(|err| {
                eprintln!("{} Error launching Android test: {}", error_tag(), err);
                0
            }),
    };
    let time = start.elapsed();
    println!("\n\nTest suite runned successfully");
    println!("    Actions executed: {}", steps_count);
    println!("    Total time elapsed: {:.2} seconds", time.as_secs_f64());
    Ok(())
}
