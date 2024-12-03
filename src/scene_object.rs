use crate::data_objects::MeshData;
use crate::scene_geo;
use nalgebra_glm::{self as glm, vec3};
use nalgebra_glm::{Mat4, Vec3};

// Add these helper functions at the module level
fn get_line_normal_from_two_points(p1: Vec3, p2: Vec3, face_normal: Vec3) -> Vec3 {
    let line = p2 - p1;
    glm::normalize(&glm::cross(&face_normal, &line))
}

fn lq(q: Vec3, p: Vec3, n: Vec3) -> f32 {
    glm::dot(&(q - p), &n)
}

fn intersect_ray_with_plane(
    ray_origin: Vec3,
    ray_direction: Vec3,
    point_on_plane: Vec3,
    plane_normal: Vec3,
) -> Option<(Vec3, f32)> {
    let denom = glm::dot(&plane_normal, &ray_direction);
    if denom.abs() < 1e-6 {
        return None;
    }

    let t = glm::dot(&(point_on_plane - ray_origin), &plane_normal) / denom;
    if t < 0.0 {
        return None;
    }

    let intersection_point = ray_origin + ray_direction * t;
    Some((intersection_point, t))
}

fn transform_point(matrix: &Mat4, point: &Vec3) -> Vec3 {
    let transformed = matrix * glm::vec4(point.x, point.y, point.z, 1.0);
    glm::vec3(transformed.x, transformed.y, transformed.z)
}

fn transform_vector(matrix: &Mat4, vector: &Vec3) -> Vec3 {
    let transformed = matrix * glm::vec4(vector.x, vector.y, vector.z, 0.0);
    glm::vec3(transformed.x, transformed.y, transformed.z)
}

#[derive(Debug)]
pub struct SceneObject {
    pub mesh_data: MeshData,
    pub pos: Vec3,
    pub rot: Vec3,
    pub scale: Vec3,
    pub min: Vec3,
    pub max: Vec3,
}
#[derive(Debug)]
pub struct Line {
    pub start: Vec3,
    pub end: Vec3,
    pub color: Vec3,
}

impl SceneObject {
    pub fn new(mesh_data: MeshData, pos: Vec3, rot: Vec3, scale: Vec3) -> Self {
        let mut out = Self {
            mesh_data,
            pos,
            rot,
            scale,
            min: Vec3::zeros(),
            max: Vec3::zeros(),
        };

        out.recalculate_min_max();
        out
    }
    pub fn recalculate_min_max(&mut self) {
        let transform = self.get_transform();
        self.min = transform_point(&transform, &self.mesh_data.min);
        self.max = transform_point(&transform, &self.mesh_data.max);
    }

    pub fn new_cube(center: Vec3, width: f32, color: Vec3) -> Self {
        let mut colors = scene_geo::CUBE_COLORS.to_vec();
        for i in 0..colors.len() / 3 {
            colors[i * 3] = color.x;
            colors[i * 3 + 1] = color.y;
            colors[i * 3 + 2] = color.z;
        }

        let mesh_data = MeshData::new(
            scene_geo::CUBE_VERTICES.to_vec(),
            scene_geo::CUBE_INDICES.to_vec(),
            colors,
            scene_geo::CUBE_NORMALS.to_vec(),
        );

        Self::new(
            mesh_data,
            center,
            vec3(0.0, 0.0, 0.0),
            vec3(width / 2.0, width / 2.0, width / 2.0),
        )
    }

    pub fn get_transform(&self) -> Mat4 {
        let mut model = glm::identity::<f32, 4>();
        model = glm::translate(&model, &self.pos);
        model = glm::rotate(&model, self.rot.x, &glm::vec3(1.0, 0.0, 0.0));
        model = glm::rotate(&model, self.rot.y, &glm::vec3(0.0, 1.0, 0.0));
        model = glm::rotate(&model, self.rot.z, &glm::vec3(0.0, 0.0, 1.0));
        model = glm::scale(&model, &self.scale);
        model
    }

    pub fn intersection(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<(Vec3, Vec3, f32)> {
        // create transformation matrix for the object
        let translation = glm::translate(&glm::Mat4::identity(), &self.pos);
        let rotation_x = glm::rotate_x(&glm::Mat4::identity(), self.rot.x);
        let rotation_y = glm::rotate_y(&glm::Mat4::identity(), self.rot.y);
        let rotation_z = glm::rotate_z(&glm::Mat4::identity(), self.rot.z);
        let scale = glm::scale(&glm::Mat4::identity(), &self.scale);

        let transform = translation * rotation_z * rotation_y * rotation_x * scale;
        let inv_transform = glm::inverse(&transform);

        // transform ray to object space
        let transformed_origin = transform_point(&inv_transform, &ray_origin);
        let transformed_direction = transform_vector(&inv_transform, &ray_direction);

        let mut smallest_t = f32::INFINITY;
        let mut found_intersection = false;
        let mut result_point = Vec3::zeros();
        let mut result_normal = Vec3::zeros();

        // iterate through triangles
        for face_idx in (0..self.mesh_data.indices.len()).step_by(3) {
            let v1_idx = self.mesh_data.indices[face_idx] as usize * 3;
            let v2_idx = self.mesh_data.indices[face_idx + 1] as usize * 3;
            let v3_idx = self.mesh_data.indices[face_idx + 2] as usize * 3;

            let v1 = glm::vec3(
                self.mesh_data.vertices[v1_idx],
                self.mesh_data.vertices[v1_idx + 1],
                self.mesh_data.vertices[v1_idx + 2],
            );
            let v2 = glm::vec3(
                self.mesh_data.vertices[v2_idx],
                self.mesh_data.vertices[v2_idx + 1],
                self.mesh_data.vertices[v2_idx + 2],
            );
            let v3 = glm::vec3(
                self.mesh_data.vertices[v3_idx],
                self.mesh_data.vertices[v3_idx + 1],
                self.mesh_data.vertices[v3_idx + 2],
            );

            let line1 = v2 - v1;
            let line2 = v3 - v1;
            let face_normal = glm::normalize(&glm::cross(&line1, &line2));

            if let Some((intersection_point, t)) =
                intersect_ray_with_plane(transformed_origin, transformed_direction, v1, face_normal)
            {
                // check if point is inside triangle using edge normals
                let edge_1_normal = get_line_normal_from_two_points(v1, v2, face_normal);
                if lq(intersection_point, v1, edge_1_normal) < 0.0 {
                    continue;
                }

                let edge_2_normal = get_line_normal_from_two_points(v2, v3, face_normal);
                if lq(intersection_point, v2, edge_2_normal) < 0.0 {
                    continue;
                }

                let edge_3_normal = get_line_normal_from_two_points(v3, v1, face_normal);
                if lq(intersection_point, v3, edge_3_normal) < 0.0 {
                    continue;
                }

                if t >= 10.0 * f32::EPSILON && t < smallest_t {
                    smallest_t = t;
                    result_point = intersection_point;
                    result_normal = face_normal;
                    found_intersection = true;
                }
            }
        }

        if found_intersection {
            let world_point = transform_point(&transform, &result_point);

            let normal_transform = glm::transpose(&inv_transform);
            let world_normal = glm::normalize(&transform_vector(&normal_transform, &result_normal));

            let world_t = smallest_t * glm::length(&self.scale);

            Some((world_point, world_normal, world_t))
        } else {
            None
        }
    }
}
