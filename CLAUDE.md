# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Stroop is a virtual machine project with research documentation on bytecode formats from various JavaScript engines and WebAssembly.

## Build Commands

```bash
cargo build                  # Build all crates
cargo test                   # Run all tests
cargo test -p stroop-vm      # Run tests for specific crate
cargo clippy                 # Run linter
cargo run -p stroop-cli      # Run the CLI
```

## Workspace Structure

```
crates/
  stroop-bytecode/       # Bytecode definitions and encoding
  stroop-text-assembly/  # SAT (Stroop Assembly Text) parser
  stroop-vm/             # Virtual machine implementation
  stroop-cli/            # Command-line interface (binary: stroop)
examples/                # Example .sat assembly files
research/                # Bytecode research (see research/CLAUDE.md)
```

## SAT File Format

The project uses `.sat` (Stroop Assembly Text) files as the human-readable assembly format. See `examples/` for samples.
