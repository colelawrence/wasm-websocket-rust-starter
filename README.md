# WASM Pathfinder Starter

A starter project that uses Rust/WASM with `wasm-bindgen` and `petgraph` for shortest path computation, and D3.js for visualization.

## Prerequisites

Install `mise` and `wasm-pack`:
```bash
# Install mise
curl https://mise.run | sh

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

Or with Homebrew:
```bash
brew install mise wasm-pack
```

## Setup

1. Install dependencies:
```bash
bun install
```

2. Start development (builds WASM and runs dev server):
```bash
bun run dev
# or directly: mise run dev
```

## Available Commands

### Using Tilt (Recommended)
- `mise run tilt` or `tilt up` - Start all development resources (WASM build + dev server)
- `mise run tilt:down` or `tilt down` - Stop all resources

### Using mise directly
- `bun run dev` or `mise run dev` - Build WASM and start dev server
- `bun run build` or `mise run build` - Production build
- `bun run wasm:watch` - Watch Rust files and rebuild WASM on changes
- `mise run wasm:dev` - Build WASM once (dev mode)
- `mise run wasm:build` - Build WASM once (production)

## Tilt Resources

The Tiltfile sets up three local resources:
1. **wasm-build** - Watches Rust files and rebuilds WASM on changes
2. **vite-server** - Runs the Vite dev server (depends on wasm-build)
3. **typecheck** - TypeScript type checking (manual trigger)

## Project Structure

- `src-rust/` - Rust WASM transport layer
- `pathfinder-core/` - Pure pathfinding logic (transport-agnostic)
- `pathfinder-server/` - WebSocket transport server
- `shared-types/` - Protocol definitions and router infrastructure
- `src/` - React/TypeScript frontend
- `src/router/` - Transport-agnostic router with WASM and WebSocket adaptors
- `src/pathfinder.tsx` - D3.js visualization with transport switching
- `pkg/` - Generated WASM module (auto-generated)
- `docs/` - Project documentation
  - `architecture/` - System design and crate structure
  - `development/` - Codegen and development guides
  - `features/` - Feature documentation
  - `proposals/` - Design proposals and RFCs

## How It Works

1. Rust computes the shortest path between points using Dijkstra's algorithm from the `petgraph` library
2. The result is serialized and sent to JavaScript via wasm-bindgen
3. D3.js visualizes the graph and highlights the shortest path
4. Points are draggable, and the path recalculates in real-time
