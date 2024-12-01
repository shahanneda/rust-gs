use std::collections::HashSet;

use crate::data_objects::MeshData;
use crate::oct_tree::{OctTreeNode, OctTreeSplat};
use crate::scene_object::SceneObject;
use crate::{data_objects::SplatData, oct_tree::OctTree};
use nalgebra_glm as glm;
use nalgebra_glm::{vec3, Vec3};

// #[derive(Serialize, Deserialize, Debug)]
pub struct Scene {
    pub splat_data: SplatData,
    pub objects: Vec<SceneObject>,
    pub line_mesh: SceneObject,
    pub light_pos: Vec3,
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
            light_pos: vec3(1.0, -3.0, 0.0),
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

    // pub fn object_at_point(&self, point: Vec3) -> Option<&SceneObject> {
    //     for object in &self.objects {
    //         // if glm::distance(&object.pos, &point) < 1.0 {
    //         //     return Some(object);
    //         // }
    //         // if object.is_point_in_object(point) {
    //         //     return Some(object);
    //         // }

    //     }
    //     None
    // }
    pub fn is_point_in_shadow(&self, point: Vec3, light_pos: Vec3) -> bool {
        let ray_origin = point;
        let dir = light_pos - point;
        let ray_direction = dir.normalize();

        let number_of_iterations = 100;
        // let dir_amount = dir / number_of_iterations as f32;

        for object in &self.objects {
            let mut intersection = Intersection {
                intersection_point: vec3(0.0, 0.0, 0.0),
                normal: vec3(0.0, 0.0, 0.0),
                t: 0.0,
            };

            if object.intersection(ray_origin, ray_direction).is_some() {
                return true;
            }

            // if sphere_intersection(
            //     ray_origin,
            //     ray_direction,
            //     object.pos,
            //     1.0,
            //     &mut intersection,
            // ) {

            //     // return true;
            // }
        }

        // for t in 0..number_of_iterations {
        //     let test_point = point + (t as f32) * dir_amount;

        // if self.object_at_point(test_point).is_some() {
        //     return true;
        // }
        // }
        false
    }

    pub fn find_shadow_splats(&mut self, node: &OctTreeNode, out_set: &mut HashSet<usize>) {
        let pos = node.center;
        let min_splats = 10;

        // if this node is not fine grain enough, try to go deeper first
        if node.splats.len() > min_splats && node.children.len() > 0 {
            for child in &node.children {
                self.find_shadow_splats(&child, out_set);
            }
            return;
        }

        // if this node is fine grain enough, just check if the center is in shadow
        if self.is_point_in_shadow(pos, self.light_pos) {
            for splat in &node.splats {
                // if self.is_point_in_shadow(vec3(splat.x, splat.y, splat.z), self.light_pos) {
                out_set.insert(splat.index);
                // }
            }
            // TODO: now actually check each individual splat
        }
    }
    pub fn calculate_shadows(&mut self, oct_tree: &OctTree) {
        let mut shadow_splats = HashSet::new();
        self.find_shadow_splats(&oct_tree.root, &mut shadow_splats);
        for index in shadow_splats {
            self.splat_data.splats[index].r -= 0.4;
            self.splat_data.splats[index].g -= 0.4;
            self.splat_data.splats[index].b -= 0.4;
        }

        // let shadow_points: Vec<_> = self
        //     .splat_data
        //     .splats
        //     .iter()
        //     .map(|splat| self.is_point_in_shadow(vec3(splat.x, splat.y, splat.z), self.light_pos))
        //     .collect();

        // for (splat, is_shadowed) in self.splat_data.splats.iter_mut().zip(shadow_points) {
        //     if is_shadowed {
        //         // splat.opacity = 0.0;
        //         splat.r -= 0.4;
        //         splat.g -= 0.4;
        //         splat.b -= 0.4;
        //     }
        // }
    }
}
// bool sphere_intersection(glm::vec3 ray_origin, glm::vec3 ray_direction,
//                         glm::vec3 sphere_pos, double sphere_radius,
//                         Intersection& intersection) {
// 	glm::vec3 tmp = ray_origin - sphere_pos;
// 	double a = glm::dot(ray_direction, ray_direction);
// 	double b = 2.0 * glm::dot(ray_direction, tmp);
// 	double c = glm::dot(tmp, tmp) - sphere_radius * sphere_radius;

// 	double discriminant = b * b - 4 * a * c;
// 	if (discriminant >= 0) {
// 		double roots[2];
// 		quadraticRoots(a, b, c, roots);
// 		double smaller_root = roots[0] < roots[1] ? roots[0] : roots[1];
// 		double larger_root = roots[0] > roots[1] ? roots[0] : roots[1];

// 		float root = smaller_root;
// 		if (smaller_root >= 0) {
// 			root = smaller_root;
// 		} else if (larger_root >= 0) {
// 			root = larger_root;
// 		} else {
// 			return false;
// 		}
// 		intersection.intersection_point = ray_origin + root * ray_direction;
// 		intersection.normal = glm::normalize(intersection.intersection_point - sphere_pos);
// 		intersection.t = root;
// 		if (intersection.t > 1000*EPSILON) {
// 			return true;
// 		}
// 	}
// 	return false;
// }

pub fn sphere_intersection(
    ray_origin: Vec3,
    ray_direction: Vec3,
    sphere_pos: Vec3,
    sphere_radius: f32,
    intersection: &mut Intersection,
) -> bool {
    let tmp = ray_origin - sphere_pos;
    let a = glm::dot(&ray_direction, &ray_direction);
    let b = 2.0 * glm::dot(&ray_direction, &tmp);
    let c = glm::dot(&tmp, &tmp) - sphere_radius * sphere_radius;

    let discriminant = b * b - 4.0 * a * c;
    if discriminant >= 0.0 {
        let root1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let root2 = (-b + discriminant.sqrt()) / (2.0 * a);

        let root = if root1 >= 0.0 {
            root1
        } else if root2 >= 0.0 {
            root2
        } else {
            return false;
        };

        intersection.intersection_point = ray_origin + root * ray_direction;
        intersection.normal = (intersection.intersection_point - sphere_pos).normalize();
        intersection.t = root;

        const EPSILON: f32 = 1e-5;
        if intersection.t > 1000.0 * EPSILON {
            return true;
        }
    }
    false
}

pub struct Intersection {
    pub intersection_point: Vec3,
    pub normal: Vec3,
    pub t: f32,
}
