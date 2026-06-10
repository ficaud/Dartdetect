# Architecture diagram notes

## Change: unified styling for data-source entities

**What:** Added a `dataSource` CSS class in the mermaid diagram inside `docs/ARCHITECTURE.md` and applied it to `dart_sensors_uart`, `dart_sensors_espnow`, and `dart_simu_cli`.

**Why:** These three entities all serve the same role — they are data sources that feed raw input (timings or coordinates) into `dart_interface`. They differ only in transport method (UART, WiFi, CLI). Giving them the same visual style communicates they are the same kind of entity.

**How:** Introduced `classDef dataSource fill:#e8f5e9,stroke:#2e7d32,stroke-width:2px` and appended `:::dataSource` to each of the three node definitions.
