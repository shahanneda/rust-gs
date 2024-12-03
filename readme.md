## CS488 Final Project - Gaussian Splatting Viewer

To run the project, I recommend simply using my deployed website:
https://shahan.ca/cs488-final-project/


You can switch between the two scenes by clicking the dropdown menu in the top left.

### Running Locally (not recomended)
- Install Cargo and Rust (https://doc.rust-lang.org/cargo/getting-started/installation.html)
- Install WasmPack (https://rustwasm.github.io/wasm-pack/installer/)
- Run `./build.sh`
- Then Open up `index.html` in your browser using a web server (I recommend using VSCode's Live Server extension)

- (to compress your own splat (from a .ply file), edit `src/local/local.rs` and run `./buildLocal.sh`)

- Uses https://github.com/rkyv/rkyv to serialize splats super fast

Credits:
For the splat rendering shaders, took some inspiration from many different online viewers:
https://github.com/playcanvas/supersplat
https://github.com/antimatter15/splat
https://github.com/kishimisu/Gaussian-Splatting-WebGL
https://github.com/mkkellogg/GaussianSplats3D
https://github.com/nerfstudio-project/gsplat

Teapot
https://github.com/kevinroast/phoria.js/blob/master/teapot.obj