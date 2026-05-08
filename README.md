# GameBench.Fyrox

Rust/Fyrox implementation of the Kaiju 2D Benchmark described in `PLAN-GAME.md`.

## Run

Rust should already be installed.

```powershell
cargo run --package executor --release
```

The game opens a fixed `1280x960` desktop window.

## Controls

Menu:

- `Enter` or `Start Benchmark`: start benchmark
- `Q` or `Quit`: exit

Benchmark:

- `A`: add 100 target objects
- `Z`: remove 100 target objects
- `R`: reset to 100 target objects
- `Q`: exit

## Development Checks

```powershell
cargo check --package gamebench_fyrox --package executor
```
