# Communication protocol

This document defines the communication protocol between the data sources and the server.

## Data format

Two kinds of data format are used in the communication between the sensors and the server:

- **Sensor timings**: Raw timings retrieved from the sensors, either through the UART or the ESP-NOW protocol.
- **Coordinates**: The coordinates (x, y) on the dartboard, used mainly by the simulation to test the server and frontend without needing physical hardware.
