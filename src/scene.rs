use crate::data_objects::MeshData;
use crate::scene_object::SceneObject;
use crate::{data_objects::SplatData, oct_tree::OctTree};
use nalgebra_glm::{vec3, Vec3};

// #[derive(Serialize, Deserialize, Debug)]
pub struct Scene {
    pub splat_data: SplatData,
    pub objects: Vec<SceneObject>,
    pub line_mesh: SceneObject,
}

impl Scene {
    pub fn new(splat_data: SplatData) -> Self {
        Self {
            splat_data,
            objects: Vec::new(),
            line_mesh: SceneObject::new(
                MeshData::new(vec![], vec![], vec![], vec![]),
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 0.0),
                vec3(1.0, 1.0, 1.0),
            ),
            // line_verts: Vec::new(),
            // line_colors: Vec::new(),
        }
    }

    pub fn add_line(&mut self, start: Vec3, end: Vec3, color: Vec3) {
        self.line_mesh.mesh_data.vertices.push(start.x);
        self.line_mesh.mesh_data.vertices.push(start.y);
        self.line_mesh.mesh_data.vertices.push(start.z);

        self.line_mesh.mesh_data.vertices.push(end.x);
        self.line_mesh.mesh_data.vertices.push(end.y);
        self.line_mesh.mesh_data.vertices.push(end.z);

        self.line_mesh.mesh_data.colors.push(color.x);
        self.line_mesh.mesh_data.colors.push(color.y);
        self.line_mesh.mesh_data.colors.push(color.z);

        self.line_mesh.mesh_data.colors.push(color.x);
        self.line_mesh.mesh_data.colors.push(color.y);
        self.line_mesh.mesh_data.colors.push(color.z);

        self.line_mesh.mesh_data.normals.push(0.0);
        self.line_mesh.mesh_data.normals.push(0.0);
        self.line_mesh.mesh_data.normals.push(1.0);

        self.line_mesh.mesh_data.normals.push(0.0);
        self.line_mesh.mesh_data.normals.push(0.0);
        self.line_mesh.mesh_data.normals.push(1.0);
    }
    pub fn clear_lines(&mut self) {
        self.line_mesh.mesh_data.vertices.clear();
        self.line_mesh.mesh_data.colors.clear();
    }

    pub fn redraw_from_oct_tree(&mut self, oct_tree: &OctTree, only_clicks: bool) {
        self.clear_lines();
        let lines = oct_tree.get_lines(only_clicks);
        for line in lines {
            self.add_line(line.start, line.end, line.color);
        }
    }
}
