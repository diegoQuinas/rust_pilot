use std::{env, fs::File, io::Read, time::Instant};

use maestro_rs::{android::*, ios::*, shared::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Obtener los argumentos de la línea de comandos
    let args: Vec<String> = env::args().collect();

    // Verificar si el archivo fue pasado como argumento
    if args.len() < 2 {
        println!("Test file path needed as argument");
        return Ok(());
    }

    let file_path = &args[1];

    println!("Test file path: {}", file_path);
    // Load the YAML file
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Deserialize the YAML into our TestFile struct
    let test_file: TestFile = serde_yaml::from_str(&contents)?;

    let start = Instant::now();
    let steps_count = match test_file.platform {
        Platform::Ios {
            capabilities,
            steps,
        } => launch_ios_main(capabilities, steps).await.unwrap(),
        Platform::Android {
            capabilities,
            steps,
        } => launch_android_main(capabilities, steps).await.unwrap(),
    };
    let time = start.elapsed();
    println!("\n\nTest suite runned successfully");
    println!("    Actions executed: {}", steps_count);
    println!("    Total time elapsed: {:.2} seconds", time.as_secs_f64());
    Ok(())
}
