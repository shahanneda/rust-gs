## Rust + WebAssembly + WebGL Gaussian Splatting Viewer

https://shahan.ca/rust-gs

- Written from scratch using rust & web assembly
- Uses counting sort to sort splats
- Custom Ply file reader
- Each splat is instance rendered on top of a rectangle which always faces the camera.
- Uses a WebGL2 data texture to send splat data to shader
- Splats are stored in an OctTree, which can be turned on and off, and used for editing, deleting, and adding splats.
- Experimental support for shadows
- Polygon based objects can be added to the scene, and blended with the splats.
- Uses https://github.com/rkyv/rkyv to serialize splats super fast
- Multiple splats can be combined



### Using the project:

Use WASD to move around, and click and drag the mouse to rotate the camera.
Hold down alt, and then click and drag the mouse to delete gaussians.
NOTE: The scenes take anywhere from 10 seconds to 60 seconds to load, depending on the scene, so please be patient! If nothing is showing up after 60 seconds, please check the console to see the error.

To move the polygon based objects, click on them to show the gizmo, and then click and drag on an axis to move the object. While holding down the button, dragging left and right will move the object along the x or z axis. For the y axis, moving the mouse up and down will move the object.

You can switch between different scenes by clicking the dropdown menu in the top left.


### Running Locally
- Install Cargo and Rust (https://doc.rust-lang.org/cargo/getting-started/installation.html)
- Install WasmPack (https://rustwasm.github.io/wasm-pack/installer/)
- Run `./build.sh`
- Then Open up `index.html` in your browser using a web server (I recommend using VSCode's Live Server extension)
- (to compress your own splat (from a .ply file), edit `src/local/local.rs` and run `./buildLocal.sh`)

Credits:
For the splat rendering shaders, took some inspiration from many different online viewers:
https://github.com/playcanvas/supersplat
https://github.com/kishimisu/Gaussian-Splatting-WebGL
https://github.com/nerfstudio-project/gsplat
