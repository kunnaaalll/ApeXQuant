# RESILIENCY ENGINE

## Overview
Measures how quickly the market recovers (spread and depth) following a liquidity shock or sweep.

## States
- `Fast`: Recovery <= 10 ms
- `Normal`: Recovery <= 50 ms
- `Slow`: Recovery <= 200 ms
- `Broken`: Recovery > 200 ms

## Usage
Resiliency directly impacts the overall Microstructure Score. A broken resiliency indicates severe liquidity dislocation.
