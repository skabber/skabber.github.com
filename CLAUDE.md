# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Personal site at skabber.com — a Dioxus 0.6 web app that renders animated ASCII "wave art" to a `<canvas>` via WebGL 2. Deployed to GitHub Pages via a GitHub Actions workflow that publishes the `trunk build` output as a Pages artifact.

## Commands

Dev shell is provided by `flake.nix` (direnv auto-loads via `.envrc`). All commands assume you're inside it.

- `trunk serve --open` — hot-reload dev server
- `trunk build --release` — optimised WASM build (outputs to `dist/`)
- `cargo check --target wasm32-unknown-unknown` — fast type-check without bundling
- `cargo clippy --target wasm32-unknown-unknown` — lints

No tests.

## Toolchain constraints (NixOS, local dev)

- `wasm-bindgen` in `Cargo.toml` is pinned to `=0.2.114` to match the `wasm-bindgen-cli` nixpkgs provides. Trunk fails loudly if they diverge. Bump both together or not at all.
- `Trunk.toml` forces Trunk to use the nix-provided `wasm-bindgen-cli` rather than downloading a GitHub-release binary (downloaded binaries won't run on NixOS due to the dynamic linker).
- The Rust toolchain is declared in `flake.nix` (stable + `wasm32-unknown-unknown`). Don't add a `rust-toolchain.toml`.

## Deploy (GitHub Actions → GitHub Pages)

`.github/workflows/deploy.yml` runs on every push to `main`:

1. Installs stable Rust with the `wasm32-unknown-unknown` target.
2. Installs `trunk` (via `jetli/trunk-action`).
3. `cargo install wasm-bindgen-cli --version 0.2.114` — must match the `Cargo.toml` pin exactly. The `Trunk.toml` `path = "wasm-bindgen"` override resolves it on `PATH`.
4. `trunk build --release` → `dist/`.
5. Copies root `CNAME` into `dist/CNAME` so Pages keeps the custom domain after deploy (Pages overwrites the Pages CNAME from the uploaded artifact's `CNAME` file).
6. Uploads `dist/` as a Pages artifact and deploys with `actions/deploy-pages@v4`.

Repo settings requirement: **Settings → Pages → Source = "GitHub Actions"** (not "Deploy from a branch"). The legacy `/docs` folder flow is gone.

If you bump `wasm-bindgen` in `Cargo.toml`, you must also update:
- `Trunk.toml` version field
- `flake.nix` / nixpkgs (so the local `wasm-bindgen-cli` matches)
- The version in `.github/workflows/deploy.yml`

## Architecture

All per-cell work happens on the GPU in a fragment shader; the CPU side is thin.

Files:
- `src/main.rs` — Dioxus app: signals (`time`, `dims`, `mouse_grid`, `click_pulses`), input handlers, window-resize hook, and the `use_future` frame loop.
- `src/gl.rs` — `Renderer` struct: shader compile/link, VAO/VBO setup, glyph atlas build, per-frame uniform uploads + draw call. `CHARS` lives here.
- `src/shaders.rs` — `include_str!`s the GLSL.
- `src/shaders/wave.vert` — trivial fullscreen quad.
- `src/shaders/wave.frag` — wave math + atlas sampling + HSL→RGB. This is where the real work is.

Rendering model worth understanding before editing:

- **Single fullscreen quad, one draw call per frame.** The fragment shader derives its cell from `gl_FragCoord`, evaluates `base_wave + mouse_wave + click_wave`, picks a glyph index from the `CHARS` density ramp, samples a glyph atlas, and multiplies by an HSL→RGB color computed from the wave value.
- **Y-flip convention**: `gl_FragCoord` is bottom-left-origin; the shader flips Y so `cell = (0,0)` is top-left — this matches the CPU port and keeps `u_mouse` / `u_pulses` in the same grid coords the input handlers produce. The atlas texture is uploaded with `UNPACK_FLIP_Y_WEBGL = 1` so canvas row 0 lands at V=1.
- **Glyph atlas**: built once at `Renderer::new` via an offscreen `HtmlCanvasElement` + 2D context — we `fill_text` each of the 65 `CHARS` into a grid (13 cols × N rows) and upload as an RGBA texture. Only the alpha channel is used at sample time (glyph mask); color comes entirely from the shader. This is the only surviving use of Canvas 2D.
- **Click pulses**: uploaded each frame as a `vec3[10]` uniform (`x, y, birthTime`); the shader iterates with a fixed bound and skips inactive slots.
- **Frame loop** (`use_future` in `main.rs`) ticks every 16 ms, advances `time`, prunes expired pulses, creates the `Renderer` lazily on first tick (once the canvas element exists), calls `set_viewport` when `dims` changes, then calls `Renderer::draw`.
- **Input**: `onmousemove` / `onclick` on the canvas convert element-relative pixel coords to grid coords by dividing by `cell_w` / `cell_h`. `mouse_grid` and `click_pulses` signals feed directly into uniforms.
- **Resize**: a raw `web_sys` `onresize` closure updates the `dims` signal (closure is `.forget()`-leaked for the page lifetime). Dioxus then re-renders the canvas element with new width/height attributes, which reallocates the WebGL drawing buffer — the viewport is re-applied inside the frame loop on dims change.

Signals (`time`, `dims`, `mouse_grid`, `click_pulses`) are read inside the render loop via `.peek()` to avoid subscribing the future to reactive updates — we don't want Dioxus reacting to the 60 Hz time tick.
