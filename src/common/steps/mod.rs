use std::path::Path;
use std::process;
use chrono;
use colored::Colorize;

use crate::common::models::Step;
use crate::common::utils::{get_current_indent_level, parse_test_file, set_current_indent_level};
use crate::logger::Logger;

/// Flattens a list of steps, resolving any `RunFlow` steps recursively.
pub async fn flatten_steps(
    steps: Vec<Step>,
    base_path: &Path,
    mermaid_parent_id: String,
) -> (Vec<Step>, String) {
    flatten_steps_with_indent(steps, base_path, mermaid_parent_id, 0).await
}

/// Internal implementation of flatten_steps that tracks the indentation level
async fn flatten_steps_with_indent(
    steps: Vec<Step>,
    base_path: &Path,
    mermaid_parent_id: String,
    indent_level: usize,
) -> (Vec<Step>, String) {
    let mut flattened_steps: Vec<Step> = Vec::new();
    let mut mermaid_steps = String::new();

    for step in steps {
        match step {
            Step::RunFlow { runFlow } => {
                let now = chrono::Local::now().timestamp_millis();
                let id = format!("idRunFlow{}({})", now, runFlow);
                mermaid_steps.push_str(&format!("{} --> {}\\n", mermaid_parent_id, id));
                let step_path = base_path.join(&runFlow);
                let string_path = step_path.display().to_string();
                
                // Use indented logging
                Logger::info_with_indent(format!("Loading step file {}", string_path.blue()), indent_level);

                // Verify file existence
                if !step_path.exists() {
                    Logger::error_with_indent(
                        format!("Error: File {} does not exist", step_path.display()),
                        indent_level
                    );
                    process::exit(1);
                }

                // Parse the step file
                let (_, steps) = parse_test_file(&step_path);

                // Store the current indentation level for the nested steps
                let next_indent_level = indent_level + 1;
                
                // Recursively flatten the steps from the loaded file with one more level of indentation
                // Use Box::pin to avoid infinitely sized future with recursive async calls
                let (sub_steps, mermaid_sub_steps) =
                    Box::pin(flatten_steps_with_indent(
                        steps, 
                        step_path.parent().unwrap(), 
                        id,
                        next_indent_level // Increase indent level for nested steps
                    )).await;
                
                // Add the indentation level to each step
                let mut indented_steps: Vec<Step> = Vec::new();
                for step in sub_steps {
                    // Store the indentation level in the step metadata
                    let step_with_indent = match step {
                        Step::RunFlow { runFlow } => Step::RunFlow { runFlow },
                        Step::TapOn { tapOn } => {
                            // Set the indentation level for this step
                            set_current_indent_level(next_indent_level);
                            Step::TapOn { tapOn }
                        },
                        Step::RunScript { runScript } => {
                            set_current_indent_level(next_indent_level);
                            Step::RunScript { runScript }
                        },
                        Step::InputText { inputText } => {
                            set_current_indent_level(next_indent_level);
                            Step::InputText { inputText }
                        },
                        Step::AssertVisible { assertVisible } => {
                            set_current_indent_level(next_indent_level);
                            Step::AssertVisible { assertVisible }
                        },
                        Step::AssertNotVisible { assertNotVisible } => {
                            set_current_indent_level(next_indent_level);
                            Step::AssertNotVisible { assertNotVisible }
                        },
                        Step::LaunchApp { launchApp } => {
                            set_current_indent_level(next_indent_level);
                            Step::LaunchApp { launchApp }
                        },
                        Step::Swipe { swipe } => {
                            set_current_indent_level(next_indent_level);
                            Step::Swipe { swipe }
                        },
                    };
                    indented_steps.push(step_with_indent);
                }
                
                flattened_steps.extend(indented_steps);

                let string_path = step_path.display().to_string();
                Logger::success_with_indent(
                    format!("Steps from {} loaded successfully", string_path.blue()),
                    indent_level
                );
                mermaid_steps.push_str(&mermaid_sub_steps);
            }
            step => {
                let now = chrono::Local::now().timestamp_millis();
                let step_name = format!("{:?}", step);
                let step_name: String = step_name.split_whitespace().next().unwrap().to_string();
                let node = format!("idStepName{}({})", now, step_name);
                mermaid_steps.push_str(&format!("{} --> {}\\n", mermaid_parent_id, node));
                
                // For top-level steps (indent_level == 0), we need to set the indentation level to 0
                // For nested steps, the indentation level is already set in the RunFlow branch
                if indent_level == 0 {
                    set_current_indent_level(0);
                } else {
                    set_current_indent_level(indent_level);
                }
                
                flattened_steps.push(step);
            }
        }
    }

    (flattened_steps, mermaid_steps)
}
