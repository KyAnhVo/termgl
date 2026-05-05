# Running the TermGL example programs

This directory ships with three demo programs, each fronted by a shell script in the project root:

| Script | What it runs | Backend |
|---|---|---|
| `./earth.sh` | A textured, rotating Earth sphere. | ASCII grayscale |
| `./solar-system.sh` | The Sun, eight orbiting textured planets, Saturn's ring, and orbit lines. | 24-bit color (half-block) |
| `./view-mesh.sh <name>` | A loaded `.obj` mesh next to its vertex-cluster simplification. | 24-bit color (half-block) |

---

## Prerequisites (both paths)

### Operating system
- **Linux on x86_64**, or **Windows + WSL2** (Ubuntu/Debian/etc.). The shipped pre-built binaries are 64-bit Linux ELF executables and will not run natively on macOS, Linux ARM, or plain Windows. If you're on macOS or non-x86 Linux, follow **Path A** below to rebuild from source.
- A modern terminal emulator. The color demos look best on **Ghostty**, **Alacritty**, **kitty**, or **WezTerm**, all of which support 24-bit truecolor and the half-block glyph `▀`. Stock `xterm` works but has noticeably worse color fidelity. **GNOME Terminal** and **Konsole** are also fine.
- Resize your terminal to at least ~80×24 cells before launching, and prefer a roughly square aspect ratio for the solar-system demo. Larger / smaller font sizes change effective resolution; the demos adapt to the cell count on every frame.

### Working directory
You **must** run all three scripts from the **project root** (the directory that contains `Cargo.toml`, the `examples/` folder, and the three `.sh` files). The scripts use relative paths, and the example programs themselves load textures and meshes via paths like `examples/assets/earth.jpg`, so they will fail if launched from anywhere else.

```sh
cd /path/to/termgl   # the directory containing this file
```

### Make the scripts executable (one-time)
If the shell complains that the scripts are not executable:

```sh
chmod +x earth.sh solar-system.sh view-mesh.sh
```

---

## Path A — You have `cargo` / `rustup` installed (recommended)

This path rebuilds the example binaries from source for your machine. It is the only option on macOS or non-x86 Linux.

1. Verify your toolchain:

   ```sh
   cargo --version
   rustc --version
   ```

   If these fail, install Rust from <https://rustup.rs> first and re-open your shell.

2. From the project root, build all examples in release mode:

   ```sh
   cargo build --release --examples
   ```

   This produces `target/release/examples/planet`, `target/release/examples/solar-system`, and `target/release/examples/show_mesh`, which is exactly what the `.sh` scripts expect.

3. Run any of the three scripts from the project root (see "Running the demos" below).

---

## Path B — You do **not** have `cargo` / `rustup` (use the shipped binaries)

The pre-built x86_64 Linux binaries are already inside `target/release/examples/` in the shipped directory. You only need to:

1. Confirm the binaries are present:

   ```sh
   ls target/release/examples/planet target/release/examples/solar-system target/release/examples/show_mesh
   ```

   All three files should exist. If any are missing, you must use **Path A** instead.

2. Make sure they are executable:

   ```sh
   chmod +x target/release/examples/planet \
            target/release/examples/solar-system \
            target/release/examples/show_mesh
   ```

3. Run the scripts from the project root (see "Running the demos" below).

If the binaries refuse to run with an "Exec format error" or "cannot execute binary file," your machine is not Linux x86_64 — switch to **Path A**.

---

## Running the demos

All commands assume your shell's working directory is the **project root**.

### `./earth.sh` — textured Earth, ASCII grayscale

```sh
./earth.sh
```

No arguments. Press `Ctrl+C` to exit.

### `./solar-system.sh` — full solar system, 24-bit color

```sh
./solar-system.sh
```

No arguments. Press `Ctrl+C` to exit. Use a terminal at least ~120×40 cells for a comfortable view.

### `./view-mesh.sh <name>` — load a mesh next to its simplification

This script **requires exactly one argument**, the base name (no extension, no path) of an `.obj` file under `examples/assets/`. Running it with no argument prints a usage message and exits.

Valid names that ship with the directory:

| Argument | Loads | Notes |
|---|---|---|
| `male` | `examples/assets/male.obj` | High-poly human figure; visibly slower than the others (≈2.5–3× slower per frame) and is the headline test case for the simplifier. |
| `canon` | `examples/assets/canon.obj` | Camera / cannon model. |
| `car` | `examples/assets/car.obj` | Car model. |
| `suitcase` | `examples/assets/suitcase.obj` | Suitcase model. |

Examples:

```sh
./view-mesh.sh male
./view-mesh.sh canon
./view-mesh.sh car
./view-mesh.sh suitcase
```

The demo renders the original mesh on the right (white) and the vertex-clustered simplified mesh on the left (yellow-green), rotating in lockstep. As a side effect, it writes a simplified copy of the mesh to `examples/assets/simplified_<name>.obj`. Press `Ctrl+C` to exit.

---

## Troubleshooting

- **"Permission denied" when running a `.sh` file.** Run `chmod +x earth.sh solar-system.sh view-mesh.sh` from the project root.
- **"No such file or directory: ./target/release/examples/planet" (or similar).** The pre-built binary is missing — use **Path A** to rebuild, or verify you are in the project root (`ls Cargo.toml` should succeed).
- **"Exec format error" or "cannot execute binary file".** The shipped binary is the wrong architecture for your machine. Use **Path A** to rebuild.
- **The colored output looks like garbled escape codes.** Your terminal does not support 24-bit truecolor. Switch to a modern terminal (Ghostty, Alacritty, kitty, WezTerm) or use `./earth.sh` (ASCII-only) instead.
- **The image looks stretched or squashed.** Adjust your terminal's font size or window dimensions; the renderer reads the cell count every frame, so resizing the window mid-demo is supported and will reflow the image.
- **`./view-mesh.sh: Usage: ./view-mesh.sh <argument>, arguments is male/canon/suitcase/car.`** You forgot the argument — pass one of `male`, `canon`, `car`, or `suitcase`.
