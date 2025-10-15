# Hexagonal Toroidal Mesh Visualizer (Vue 3 + Three.js + WebGL)

## Quick Start

```bash
cd /opt/castle/workspace/flagship/crates/lens-v2-node/tests/hex-viz
npm run dev
```

Open http://localhost:5173

## What You Get

- **Vue 3** with Composition API
- **Three.js WebGL** for insane performance
- **Vite** for instant hot reload
- Modular components (Header, HexRenderer, ControlPanel, Toast)

## Integration with Rust

The Rust test will serve the built app:

```bash
npm run build
# Outputs to dist/
```

Then update `configurable_mesh_test.rs` to serve from `dist/` instead of embedded HTML.
