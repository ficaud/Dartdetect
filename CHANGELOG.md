# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-06-08

### Changed
- Global restructured the project into a modular architecture to improve maintainability and scalability across components.

### Added
- `README.md`: project logo.
- `dart_engine/`: core Rust library containing modules for dart localization, scoring, and game management.
- `dart_ui/`: Svelte-based user interface that renders the dartboard and displays detected dart positions in real time.
- `dart_server/`: backend server that ingests sensor data, processes it through the `dart_calculator`, and serves results to the UI via WebSocket.
- `dart_simu_cli/`: command-line tool for simulating dart throws and testing localization algorithms without physical hardware.
- `scripts/`: utility scripts for data processing and streamlining local deployment of the `dart_server`.

## [1.0.0] - 2026-05-31

### Added

- `dart_calculator`: modules for processing sensor timing delays after a dart impact and computing the dart’s position on the board using a gradient descent algorithm.
- `dart_core`: core data structures and shared utilities used throughout the DartDetect project.
- `dart_interpretor`: modules for interpreting the localized dart impact and converting it into a score based on the detected board zone and multiplier.
