## CS488 Final Project - Gaussian Splatting Viewer

To run the project, I recommend simply using my deployed website in a browser:
https://shahan.ca/cs488-final-project/


### Using the project:

Use WASD to move around, and click and drag the mouse to rotate the camera.
Hold down alt, and then click and drag the mouse to delete gaussians.
NOTE: The scenes take anywhere from 10 seconds to 60 seconds to load, depending on the scene, so please be patient! If nothing is showing up after 60 seconds, please check the console to see the error.

To move the polygon based objects, click on them to show the gizmo, and then click and drag on an axis to move the object. While holding down the button, dragging left and right will move the object along the x or z axis. For the y axis, moving the mouse up and down will move the object.

You can switch between different scenes by clicking the dropdown menu in the top left.


### Running Locally (not recommended)
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

## Objectives

1. The scene is made of 3D gaussian splats, and there is a toggle which allows showing all the individual splats.

2. The gaussian splats are alpha blended together, and there is toggle which can turn blending on and off.

3. The scene is sorted in back to front order in realtime every time the camera view changes, and there is a toggle which turns this off for the purpose of demonstration.

4. The splats are stored in an OctTree, which can be turned on and off. You can see the performance difference of editing splats when having the OctTree and not having it, and there is a toggle to visualize the OctTree.

5. There is a delete button, which allows the user to click on a certain point on the screen, and delete gaussians close to that point.

6. You can put polygon based meshes into the scene, which are correctly blended with the gaussian splats.

7. Polygon based meshes cast shadows on to the gaussian splats.

8. Polygon based objects will not intersect with the gaussian splat, and there is some visual indication when there is a collision.

9. Two gaussian splat scenes can be combined and rendered at the same time.

10. The starting scene is interesting, having many diverse objects of different scales and colors.