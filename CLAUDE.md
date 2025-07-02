# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based WebAssembly Gaussian Splatting viewer that renders 3D scenes in the browser using WebGL2. The project combines Rust's performance with web technologies to create an interactive 3D viewer.

## Architecture

- **Dual Build Targets**: The project compiles to both WASM (for web) and native binary (for local processing)
- **Web Binary** (`src/web.rs`): Main entry point for WASM compilation, handles WebGL rendering and browser interactions
- **Local Binary** (`src/local/local.rs`): Native binary for processing PLY files and data conversion
- **Core Modules**:
  - `splat.rs` - Gaussian splat data structures and rendering logic
  - `oct_tree.rs` - Spatial data structure for efficient splat management
  - `renderer.rs` - WebGL2 rendering pipeline (WASM only)
  - `camera.rs` - Camera controls and transformations (WASM only)
  - `loader.rs` - File loading and parsing utilities
  - `ply_splat.rs` - PLY file format handling
  - `scene.rs` - Scene management and object composition

## Key Technologies

- **Serialization**: Uses `rkyv` for fast binary serialization of splat data
- **Rendering**: WebGL2 with custom shaders for Gaussian splat rendering
- **Parallelization**: Uses `rayon` and `wasm-bindgen-rayon` for multi-threading
- **UI**: `egui` for web-based GUI components
- **File Formats**: Supports PLY files and custom `.rkyv` serialized format

## Common Commands

### Building for Web
```bash
./build.sh
```
This runs `wasm-pack build --target web` and removes the generated `.gitignore` file.

### Building for Local Processing
```bash
./buildLocal.sh
```
This builds and runs the local binary for PLY file processing.

### Testing
```bash
cargo test
```

### Development Workflow
1. Use `./build.sh` to compile for web deployment
2. Serve `index.html` with a local web server (e.g., VS Code Live Server)
3. Use `./buildLocal.sh` to process PLY files into the optimized `.rkyv` format
4. Edit `src/local/local.rs` to configure local processing tasks

## Data Pipeline

- Raw 3D scans are stored as PLY files in the `splats/` directory
- Use the local binary to convert PLY files to optimized `.rkyv` format
- The web application loads `.rkyv` files for faster startup times
- Splats are organized in an octree for efficient spatial queries and rendering

## Performance Notes

- Release builds are configured with `opt-level = 0` and `wasm-opt = false` for faster compilation during development
- Uses counting sort for efficient splat depth sorting
- Implements instance rendering for splats using WebGL2 data textures