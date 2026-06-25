# Historical Feature Replay Implementation

## Overview
The Replay engine guarantees bit-for-bit deterministic state recreation for any historical timestamp, essential for backtesting and ML model training.

## Features
- **ReplayController**: Orchestrates the timeline and speed of the replay stream.
- **ReplaySpeed**: Variable time dilation (`Normal/1x`, `Fast/10x`, `Rapid/100x`, `Turbo/1000x`).
- **ReplayWindow**: Configurable start/end boundaries.
- **ReplayCheckpoint**: Sparse state snapshots ensuring rapid seek times to any historical offset.
- **ReplaySnapshot**: The full serialized event blob bound to a checkpoint for strict state restoration.
