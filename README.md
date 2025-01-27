![RustPilotLogo](./assets/RustPilotLogo.png)

![Build Status](https://github.com/diegoQuinas/RustPilot/actions/workflows/rust.yml/badge.svg)

# RustPilot

Do you need to write automated tests for Android, Ios, or Flutter very fast, stable and completely painless?

And not only that, it also writes the report, or an exportable cucumber report. (not yet developed)

And it's written in a very low level programming language making it very fast and with (normally) very deterministical results

This is the solution, as a automation engineer, and after severe years of pain and work have implemented, and I hope you enjoy using it as much as I do and feel completely free to contribute or add the features you think you need, or fork it and make your own implementations, or even use it with commercial purposes.

![gif](./assets/Showreel.gif)

### How it works

RustPilot it's an intermediary into the appium_client of rust and a yaml of instructions very similar and compatible with Maestro (automation mobile) like test instructions yaml file

### For example

```yaml
appId: org.wikipedia
tags:
  - android
---
- runFlow: 'add-language.yml'
- runFlow: 'remove-language.yml'
- tapOn: 'CONTINUE'
- assertVisible: 'New ways to explore'
- tapOn: 'CONTINUE'
- assertVisible: 'Reading lists with sync'
- tapOn: 'CONTINUE'
- assertVisible: 'Send anonymous data'
- tapOn: 'GET STARTED'
- runFlow: 'scroll-feed.yml'
- runFlow: 'perform-search.yml'
```

## Installation and use

To test and install this software, just clone the master branch and run `cargo run` with the capabilities json file path as an argument and the test file path as second argument
