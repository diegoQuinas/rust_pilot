![RustPilotLogo](./assets/RustPilotLogo.png)

[![Rust CI](https://github.com/diegoQuinas/RustPilot/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/diegoQuinas/RustPilot/actions/workflows/rust-ci.yml)
[![Code Coverage](https://codecov.io/gh/diegoQuinas/RustPilot/branch/main/graph/badge.svg)](https://codecov.io/gh/diegoQuinas/RustPilot)
[![Crates.io](https://img.shields.io/crates/v/rustpilot.svg)](https://crates.io/crates/rustpilot)
[![Rust Version](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org)

# RustPilot

## What is RustPilot?

RustPilot is a powerful, fast, and reliable tool for automating mobile application testing. It supports Android, iOS, and Flutter applications with a simple YAML-based test definition format.

**Key Features:**
- Write tests in simple, readable YAML format
- Support for Android, iOS, and Flutter applications
- Fast execution with Rust's performance benefits
- Automatic test report generation
- Deterministic test results
- Compatible with Maestro-style test instructions

![gif](./assets/Showreel.gif)

## How It Works

RustPilot acts as an intermediary between your test definitions and the Appium automation framework:

```mermaid
flowchart TD
    %% Class definitions for styling
    classDef engineer fill:#f9d5e5,stroke:#333,stroke-width:1px
    classDef yaml fill:#eeeeee,stroke:#333,stroke-width:1px
    classDef rustpilot fill:#d0f4de,stroke:#333,stroke-width:1px,color:#333
    classDef client fill:#e4c1f9,stroke:#333,stroke-width:1px
    classDef server fill:#a9def9,stroke:#333,stroke-width:1px
    classDef device fill:#fcf6bd,stroke:#333,stroke-width:1px
    
    QA[QA Engineer] -->|writes| YAML[Tests in YAML]
    YAML -->|read by| RP[RustPilot]
    RP -->|uses| ARC[Appium Rust Client]
    ARC -->|sends HTTP requests| AS[Appium Server]
    AS -->|interacts with| Android[Android Device]
    AS -->|interacts with| iOS[iOS Device]
    AS -->|interacts with| Flutter[Flutter App]
    
    %% Apply classes to nodes
    QA:::engineer
    YAML:::yaml
    RP:::rustpilot
    ARC:::client
    AS:::server
    Android:::device
    iOS:::device
    Flutter:::device
```

RustPilot reads your YAML test files (which are compatible with Maestro-style test instructions), processes them, and uses the Appium Rust client to communicate with the Appium server for executing test actions on your target devices.

## Example Test File

```yaml
appId: org.wikipedia
tags:
  - android
---
# Test steps
- runFlow: 'add-language.yml'      # Include another test file
- runFlow: 'remove-language.yml'    # Include another test file
- tapOn: 'CONTINUE'                # Tap on an element
- assertVisible: 'New ways to explore'  # Verify element is visible
- tapOn: 'CONTINUE'
- assertVisible: 'Reading lists with sync'
- tapOn: 'CONTINUE'
- assertVisible: 'Send anonymous data'
- tapOn: 'GET STARTED'
- runFlow: 'scroll-feed.yml'       # Run another test sequence
- runFlow: 'perform-search.yml'    # Run another test sequence
```

## Installation

### Prerequisites
- Rust and Cargo (latest stable version)
- Appium Server (for mobile device interaction)
- Android SDK or iOS development tools (depending on your testing targets)

### Steps

1. **Clone the repository:**
   ```bash
   git clone https://github.com/diegoQuinas/RustPilot.git
   cd RustPilot
   ```

2. **Build the project:**
   ```bash
   cargo build --release
   ```

## Usage

### Basic Usage

```bash
cargo run -- <capabilities_file.json> <test_file.yml>
```

### Capabilities File Example

```json
{
  "platformName": "Android",
  "appium:automationName": "UiAutomator2",
  "appium:deviceName": "Android Emulator",
  "appium:app": "/path/to/your/app.apk",
  "appium:noReset": true
}
```

## Test Reports

RustPilot automatically generates test reports after execution. Reports are saved in the `reports` directory and include:
- Test execution summary
- Steps executed
- Execution time
- Test details and results

## Contributing

Contributions are welcome! Feel free to:
- Report bugs
- Suggest new features
- Submit pull requests

Please follow Rust best practices when contributing code to maintain the project's modular, extensible, and readable structure.
