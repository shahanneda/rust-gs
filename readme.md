## Rust + WebAssembly + WebGL Gaussian Splatting Viewer


- Written from scratch using rust & web assembly
- Uses counting sort to sort splats
- Custom Ply file reader
- Each splat is instance rendered on top of a rectangle which always faces the camera.
- Uses a WebGL2 data texture to send splat data to shader

- Uses https://github.com/rkyv/rkyv to serialize splats super fast
- See `src/local/local.rs` to compress a splat and run `./buildLocal.sh``
To build normally, run `./build`



Credits:
For the shaders, took some inspiration from this WebGL viewer:
https://github.com/kishimisu/Gaussian-Splatting-WebGL