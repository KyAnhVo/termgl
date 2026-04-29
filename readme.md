# TERMGL
Very easy Rust crate for terminal graphics stuffs

## Goal
This project aims to create a 3D library to do small graphics stuffs in the terminal for the ricing community, heavily inspired by the legends CMatrix and Asciiquarium where I use on the daily basis to rice my Arch i3 twm.

## Interface

### Mesh
This is the most important "thing" in the project. To initiate a mesh, use Mesh::new to create an empty mesh.
```rust
let material: Material = Material { 
    specular_constant: Vec3::new(...), 
    ambient_constant: Vec3::new(...),
    specular_exponent: 50.0,
};
// For most scenes and most objects, no_shade would be false. Though for specific cases,
// like object light, or scenes where no shading is required, flip to true.
let no_shade: bool = false;
let mut mesh: Mesh = Mesh::new(material, no_shade);
```
Prefer mesh to be mutable so that we can move the mesh around.
The orientation of the mesh is defined by its `origin` and the `orthonormal_basis`, where `origin` defines its location in the world space and `orthonormal_basis` defines the orientation.
I provided some methods to act on the two values (since it cannot be changed directly):
* `Mesh::get_origin(&self)` gets the mesh's origin
* `Mesh::get_orthonormal_basis(&self)` gets the mesh's orientation
* `Mesh::rotate(&mut self, rotation: Mat3)` rotates the object's orthonormal basis by the `rotation` matrix
* `Mesh::translate(&mut self, movement: Vec3)` moves the object's origin by a `movement` vector
* `Mesh::move_origin(&mut self, movement: Mat4)` moves the object by a homogenous matrix `movment`
* `Mesh::move_origin_to(&mut self, to: Vec3)` sets to object's origin at the `to` vector
