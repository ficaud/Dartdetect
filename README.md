# Dartdetect

<p align="center">
	<img src="docs/img/dartdetect.png" alt="Dartdetect logo" width="150"/>
</p>

DartDetect estimates a dart's position on the board from sensor timing data using a distance-of-arrival model and gradient descent.

## Architecture

The project is split into four main components:

### `dart_engine`
Core Rust library that implements the physics and game logic:
- **`dart_calculator`** — processes piezo sensor timing delays to locate the dart impact point using a distance-of-arrival model and a gradient descent solver.
- **`dart_interpretor`** — converts a localized impact point into a dartboard score (sector + multiplier).
- **`dart_scoring`** — high-level scoring pipeline that combines calculation and interpretation.
- **`dart_game`** — game session management (X01, player turns, win conditions).

### `dart_server`
Axum-based WebSocket server that bridges the engine and the UI:
- Listens for impact coordinates or raw sensor timings from connected clients.
- Delegates processing to the `dart_engine` and manages game state.
- Broadcasts scores, game state, and status updates to all connected WebSocket clients in real time.

### `dart_ui`
Svelte frontend that renders the dartboard and live game data:
- Displays the dartboard SVG with sector labels, score multipliers, and impact markers.
- Shows player scores and highlights the current turn.
- Responsive layout adapting from desktop to mobile.

### `dart_simu_cli`
Command-line simulation tool for testing without physical hardware:
- Simulates sensor timings for arbitrary impact points.
