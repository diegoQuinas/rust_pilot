use appium_client::{
    capabilities::{
        android::AndroidCapabilities, AppCapable, AppiumCapability, UiAutomator2AppCompatible,
    },
    find::{AppiumFind, By},
    wait::AppiumWait,
    ClientBuilder,
};
use fantoccini::{
    actions::{InputSource, PointerAction, TouchActions, MOUSE_BUTTON_LEFT},
    Client,
};
use serde::{Deserialize, Serialize};
use std::{env, fs::File, io::Read, time::Instant};
use tokio::time::{timeout, Duration};

#[derive(Debug, Serialize, Deserialize)]
struct TestFile {
    capabilities: TestCapabilities,
    steps: Vec<Step>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestCapabilities {
    app_path: String,
    full_reset: bool,
    platform_version: String,
    custom_cap: Vec<CustomCapability>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CustomCapability {
    key: String,
    value: CustomCapabilityValue,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum CustomCapabilityValue {
    BooleanValue(bool),
    StringValue(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct Step {
    selector: ElementSelector,
    actions: Vec<Action>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Action {
    AssertVisible,
    TapOn,
    ScrollUntilVisible,
    InsertData { data: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ElementSelector {
    Text {
        text: String,
    },
    Xpath {
        xpath: String,
    },
    ClassName {
        class_name: String,
        instance: Option<i32>,
    },
}
fn set_custom_capabilities(caps: &mut AndroidCapabilities, test_file: &TestFile) {
    for custom_capability in test_file.capabilities.custom_cap.clone() {
        match custom_capability.value {
            CustomCapabilityValue::BooleanValue(value) => {
                caps.set_bool(&custom_capability.key, value)
            }
            CustomCapabilityValue::StringValue(value) => {
                caps.set_str(&custom_capability.key, &value)
            }
        }
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Obtener los argumentos de la línea de comandos
    let args: Vec<String> = env::args().collect();

    // Verificar si el archivo fue pasado como argumento
    if args.len() < 2 {
        println!("File path needed as argument");
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

    // Configure the Appium driver
    let mut caps = AndroidCapabilities::new_uiautomator();

    let app_path = test_file.capabilities.app_path.clone();
    caps.app(&app_path);

    caps.platform_version(&test_file.capabilities.platform_version);
    caps.full_reset(test_file.capabilities.full_reset);

    set_custom_capabilities(&mut caps, &test_file);

    println!("Launching app {}", test_file.capabilities.app_path);
    let client = ClientBuilder::native(caps)
        .connect("http://localhost:4723/")
        .await?;
    println!("Launched!");
    // Crear un objeto
    let mut new_steps = test_file;

    loop {
        let mut input = String::new();

        // Imprimir un mensaje pidiendo input
        println!("\"S\" for scanning, \"wq\" for write and exit");

        // Leer el input desde la entrada estándar
        std::io::stdin()
            .read_line(&mut input) // Lee la línea y la almacena en `input`
            .expect("Falló la lectura de la línea");

        // Eliminar el salto de línea (\n) que se agrega al final del input
        let input = input.trim();

        match input {
            "S" => scan(client.clone(), &mut new_steps).await,
            "wq" => {
                let file = File::create("output.yaml")?;

                serde_yaml::to_writer(file, &mut new_steps)?;
                println!("Succesfully written to output.yaml");
                return Ok(());
            }
            &_ => {
                println!("Not recognized input");
            }
        }
    }
}

async fn scan(client: Client, new_steps: &mut TestFile) {
    let texts = client
        .find_all_by(By::uiautomator(
            "new UiSelector().className(\"android.widget.TextView\");",
        ))
        .await
        .unwrap();

    for t in texts {
        let text_content = t.text().await.unwrap();
        let new_text_step = Step {
            selector: ElementSelector::Text { text: text_content },
            actions: vec![Action::AssertVisible],
        };
        new_steps.steps.push(new_text_step);
    }
}
