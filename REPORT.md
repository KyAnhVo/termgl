# TermGL — Project Report

**Course:** CS 4361.001 — Computer Graphics
**Instructor:** Dr. Xiaohu Guo
**Author:** Ky Anh Vo (solo project)
**Project title:** TermGL — A CPU rasterizer and 3D graphics library for the terminal
**Demo video:** <https://youtu.be/pOcIk4acaEc>

---

## 1. Problem Summary

The "ricing" hobby — customizing a Linux/tmux/TTY environment with small visual flourishes — has grown alongside the rise of tiling window managers (Hyprland, Sway, i3) and Wayland. Hobbyist staples like `cmatrix`, `asciiquarium`, and `cbonsai` show that there is appetite for graphical effects inside the terminal, but each is a hand-rolled one-off. There is no shared library that lets a developer drop a textured, lit, animated 3D scene into a terminal window the way OpenGL or WebGL would for a real GPU surface.

**TermGL** is a Rust crate that fills that gap: a CPU-side 3D rasterization pipeline that uses the terminal as its render target, built on top of `glam` (Rust's GLM-style linear algebra crate) with no GPU dependency. The intended consumer is a Linux/tmux user who wants a small animated 3D scene without leaving the TTY, and developers who want a simple library they can extend.

---

## 2. Description of Work

### 2.1 Library architecture

I implemented a complete fixed-function 3D pipeline. The relevant modules live in `src/graphics/`:

- **Camera & projection** (`camera.rs`) — perspective and orthographic matrices built from an up/gaze/right basis, with a `look_at` helper and an aspect-ratio update that follows the terminal size on every frame.
- **Mesh** (`mesh.rs`) — vertex / UV / normal arrays with an indexed EBO, object-to-world transform, an `.obj` + `.mtl` importer/exporter, and ear-clipping triangulation with a triangle-fan fallback.
- **Rasterizer** (`rasterizer.rs`) — a barycentric scanline rasterizer over the screen-space bounding rectangle of each triangle, with perspective-correct interpolation (1/w), back-face culling, NDC clipping, and a depth buffer.
- **Shading** (`shader.rs`, `point_light_source.rs`) — Phong (per-pixel) and Gouraud (per-vertex, interpolated) illumination paths driven by point lights with separate ambient/diffuse/specular intensities and inverse-square distance falloff.
- **Texturing** (`uv_map.rs`) — bilinearly-sampled `UVMap`, normal mapping using a TBN basis, and parallax mapping driven by a `HeightMap`.
- **Printer** (`printer.rs`) — two render backends:
  - An ASCII grayscale ramp `" .:-=+*#%@"`.
  - A 24-bit truecolor backend that uses the upper-half-block glyph `▀` plus paired foreground/background ANSI colors to fit two vertical pixels per terminal cell, doubling effective vertical resolution.
- **Pipeline** (`pipeline3d.rs`) — orchestrates a frame: resize → clear → transform meshes to NDC → rasterize → print.

### 2.2 Mesh simplification

I implemented the Rossignac & Borrel *vertex clustering* simplification algorithm in `src/simplifier/vertex_cluster.rs` (paper cited in `PAPERS.md`):

1. **Grade each vertex** by `(1 − |max cos θ|) · max_edge_length` so vertices that lie on silhouettes or anchor large faces weigh more.
2. **Partition the mesh's bounding box** into uniform cubic cells of side `hxyz`.
3. **Collapse each cell** to a single grade-weighted centroid, which becomes one vertex of the new mesh.
4. **Re-emit triangles**, dropping any that degenerated to a point or edge after the collapse.

UVs and normals are weighted-averaged by the same per-vertex grades, so a simplified mesh still texture-maps and shades reasonably without re-baking.

### 2.3 Demo applications

- `examples/planet.rs` — a single textured sphere (Earth) rendered with the ASCII grayscale backend.
- `examples/show_mesh.rs` — takes any `.obj`, runs the vertex-clustering simplifier, and renders the original (white) next to the simplified version (yellow-green) with a rotating camera. Used to qualitatively validate the simplifier.
- `examples/solar-system/` — eight planets plus the sun, Saturn's ring, and per-planet orbit lines (rendered as flat annuli). Each planet has independent rotational and orbital velocities; the sun is the scene's single point light.

### 2.4 Things I found interesting while building

- **Vertex/Triangle decoupling.** The mesh data structure originally bundled position, color, normal, and UV into a single `MeshVertex`, with each triangle holding three of those by value. The current design (commits `1bff16c` *refactor for uv mapping and decoupling vertex with color and normal and uv* and `d425582` *change MeshVertexIndices to VertexIndices*) splits these into three flat arrays — `vertices`, `normals`, `uv` — and represents each face as three `VertexIndices = (vertex_ind, normal_ind, uv_ind)` tuples that index into them independently. This mirrors both OpenGL's VBO/EBO layout and the on-disk `.obj` `v/vt/vn` format exactly, which collapses the importer to almost nothing; it also lets one position participate in multiple faces with different normals (sharp edges) or different UVs (seams) without duplicating geometry. The original bundled representation made every seam a duplicated vertex.
- **The half-block `▀` trick.** Each terminal cell is rendered as a single upper-half-block glyph with the foreground color set to the top pixel and the background color set to the bottom pixel. That one character buys 2× vertical resolution and is the entire reason the colored output looks like graphics rather than ASCII art.

### 2.5 Dead ends and major challenges

- **More sophisticated simplification algorithms (QEM and SEAMLESS).** Before settling on Rossignac–Borrel vertex clustering, I tried and actually implemented two more complex approaches: quadric error metrics (QEM, an edge-collapse algorithm driven by per-vertex error quadrics) and the SEAMLESS algorithm (seam-aware mesh processing, which characterizes seam-free textures as the null space of a linear operator and gives topological/geometric conditions under which an edge collapse preserves a UV parameterization without introducing cracks). Both were aimed at producing visually cleaner output, especially around UV seams on textured meshes. At terminal resolution, however, the half-block color printer simply does not have enough pixels to show the difference — the visual distinction between QEM / SEAMLESS output and plain vertex clustering was too small to register on screen. The whole ordeal was scratched in favor of the simpler, faster, more robust clustering algorithm; the QEM removal is recorded in commit `946e37d removed qem (for the meantime)`.
- **Closure-free input system.** Proposal goal 6 was an input/event system. Every clean Rust API I sketched eventually went through `Box<dyn Fn>` or trait objects, and the indirection cost was measurable on a CPU pipeline running at 20–30 FPS. Rather than ship a feature that visibly hurt frame time, I cut the input system entirely.
- **Ear-clipping robustness.** Real-world `.obj` files contain near-degenerate or non-strictly-convex polygons that cause the ear-clipping inner loop to stall with no ears left. I added a stall-detection path that falls back to triangle-fan triangulation. The fan version is uglier on concave faces, but the importer never refuses a mesh.
- **CPU shadow mapping.** A post-proposal stretch goal. Shadow mapping costs roughly one additional full rasterization pass per light; even with only two lights this tripled per-frame work and dropped the solar-system demo below interactive rates. Cut.

---

## 3. Results

The final deliverable is a working Rust crate (`termgl`) plus three runnable examples. The composite figure `report-assets/results.png` shows simultaneous captures of all three demos:

- **Top-left** — `solar-system`: the sun, eight orbiting textured planets, Saturn's ring, and orbit lines, rendered with the half-block 24-bit color backend.
- **Top-right and middle-right** — `show_mesh` on `male.obj` and `canon.obj`. The original mesh (white) is rendered side-by-side with its vertex-clustered simplification (green), rotating in lockstep so the loss of detail is directly visible.
- **Bottom-left** — `planet.rs`: a textured Earth sphere using the ASCII grayscale backend, demonstrating texture sampling at low effective resolution.
- **Bottom-right** — `show_mesh` on `car.obj`, again original vs. simplified.

A live demonstration of all three programs is on YouTube: <https://youtu.be/pOcIk4acaEc>.

Concretely, the library today supports:

- Arbitrary `.obj` import (multi-material `.mtl` is honored) and round-trip export.
- Texture, normal, and height maps with bilinear sampling and parallax UV correction.
- Phong and Gouraud shading, multiple point lights per scene.
- Runtime terminal resize — the pipeline reads the terminal size each frame and rebuilds its buffers if the user has resized the window.
- Two render backends: ASCII grayscale (`PrinterType::Ascii`) and 24-bit half-block color (`PrinterType::Color`), selected by the caller.

The unsimplified `male.obj` runs visibly slower than the other meshes — roughly **2.5–3× the per-frame cost** of `canon.obj` or `car.obj` — which is precisely the motivation for shipping the vertex-cluster simplifier in the same crate.

---

## 4. Analysis

### 4.1 Original proposal goals

| # | Goal | Status |
|---|---|---|
| 1 | Triangle / projection / rasterization pipeline with ASCII grayscale output | **Done** |
| 2 | Half-block colored rendering at doubled vertical resolution | **Done** — `PrinterType::Color` using `▀` and 24-bit ANSI |
| 3 | Point-light lighting system | **Done** — both Phong and Gouraud paths, multiple lights |
| 4 | *(TBD)* Mesh light system | **Partial** — `PointLightSource` carries an optional `wrapper_mesh` so a renderable mesh can stand in for a point light (the sun uses this via `no_shade=true`), but a real area / mesh-light integration was not implemented. |
| 5 | *(TBD)* Border rasterizer / high-pass filter | **Not done** |
| 6 | *(TBD)* Input system | **Not done** — see "closure-free input" above |

Beyond the proposal I added: full `.obj` / `.mtl` import + export, ear-clipping triangulation with fan fallback, parallax + normal mapping with a TBN basis, the Rossignac–Borrel vertex-clustering simplifier, and the `show_mesh` example that lets the simplifier be evaluated visually on arbitrary input meshes.

### 4.2 Where I fell short

- **No shadows.** CPU cost made shadow mapping infeasible at interactive rates even with only two lights.
- **No `crates.io` release.** Public-API documentation is thin and the crate is not yet ready for outside consumers.
- **No parallelism.** The rasterizer is single-threaded. Rayon over per-triangle or per-tile work, or `tokio` for async I/O on the printer side, would likely give a large speedup — particularly on the human mesh — but I did not get to it.
- **Limited built-in primitives.** `create_sphere` and `create_ring` cover the planet/orbit demo and not much else; shearing/affine helpers and a richer primitive set would have made the library noticeably more general.
- **No physics demo.** I wanted at least a small rigid-body or n-body demo running on top of the renderer; out of time.
- **Input system shipped as nothing.** Goal 6 was TBD in the proposal, but I'd rather have shipped *some* form of input than none.

### 4.3 Where I exceeded the proposal

Mesh I/O, mesh simplification, and the full texturing stack (bilinear sampling + normal mapping + parallax mapping) were not in the original plan. The `show_mesh` demo, in particular, lets the simplifier be evaluated qualitatively on any user-supplied `.obj`, which I think is the most interesting result in the project beyond the headline solar-system demo.

### 4.4 Overall

I met all three firm proposal goals (1–3) and roughly half of the stretch goals (4 partially, 5 and 6 not at all), while adding substantial work on mesh I/O, simplification, and advanced texturing that was not planned. The library is usable today as a drop-in CPU 3D pipeline for terminal apps, which was the original vision. The biggest remaining gap between intent and result is **performance**: a release-quality terminal graphics library would need to handle a ~50k-triangle character mesh at 30+ FPS, and TermGL does not — yet. Parallelism and a tighter rasterizer inner loop are the obvious next steps.

---

## References

- Rossignac, J. R. and Borrel, P. *Multi-resolution 3D Approximations for Rendering Complex Scenes*. In *Modeling in Computer Graphics: Methods and Applications*, pp. 455–465. <https://faculty.cc.gatech.edu/~jarek/papers/VertexClustering.pdf>
- `glam` linear algebra crate — <https://crates.io/crates/glam>
- `crossterm` terminal-control crate — <https://crates.io/crates/crossterm>
- `image` crate (texture loading) — <https://crates.io/crates/image>

