use crate::log;
use crate::splat::Splat;
use nalgebra_glm::{vec3, Vec3};

use crate::scene_object::{Line, SceneObject};
use nalgebra_glm as glm;

#[derive(Clone, Debug)]
pub struct OctTreeSplat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub opacity: f32,
    pub index: usize,
}

impl From<&Splat> for OctTreeSplat {
    fn from(splat: &Splat) -> Self {
        OctTreeSplat {
            x: splat.x,
            y: splat.y,
            z: splat.z,
            opacity: splat.opacity,
            index: 0, // Will be set properly during construction
        }
    }
}

#[derive(Debug)]
pub struct OctTreeNode {
    pub children: Vec<OctTreeNode>,
    pub splat_indices: Vec<usize>,
    pub center: Vec3,
    pub half_width: f32,
    pub touched: bool,
}

#[derive(Debug)]
pub struct OctTree {
    pub root: OctTreeNode,
    all_splats: Vec<Splat>,
}

// Dynamic split parameters will be chosen at runtime in `OctTree::new`.

// These are just fallback defaults for very small scenes.
const DEFAULT_SPLIT_LIMIT: usize = 1000;
const DEFAULT_MAX_DEPTH: usize = 10;

#[derive(Clone, Copy)]
struct SplitParams {
    split_limit: usize,
    max_depth: usize,
}

impl OctTreeNode {
    pub fn new(splat_indices: Vec<usize>, center: Vec3, half_width: f32, params: SplitParams, all_splats: &[Splat]) -> Self {
        let mut out = OctTreeNode {
            children: Vec::new(),
            splat_indices,
            center,
            half_width,
            touched: false,
        };

        out.propogate_splats_to_children(0, params, all_splats);

        out
    }

    pub fn new_root(splats: Vec<Splat>, center: Vec3, half_width: f32, params: SplitParams) -> (Self, Vec<Splat>) {
        let indices: Vec<usize> = (0..splats.len()).collect();
        let mut root = OctTreeNode {
            children: Vec::new(),
            splat_indices: indices,
            center,
            half_width,
            touched: false,
        };
        
        root.propogate_splats_to_children(0, params, &splats);
        (root, splats)
    }

    fn index_to_color(index: usize) -> Vec3 {
        match index {
            0 => vec3(1.0, 0.0, 0.0),
            1 => vec3(0.0, 1.0, 0.0),
            2 => vec3(0.0, 0.0, 1.0),
            3 => vec3(1.0, 1.0, 0.0),
            4 => vec3(1.0, 0.0, 1.0),
            5 => vec3(0.0, 1.0, 1.0),
            6 => vec3(0.1, 0.8, 0.7),
            7 => vec3(0.4, 0.4, 0.4),
            _ => panic!("Invalid index"),
        }
    }

    fn index_to_direction(index: usize) -> Vec3 {
        vec3(
            if (index & 0b100) == 0 { 1.0 } else { -1.0 },
            if (index & 0b010) == 0 { 1.0 } else { -1.0 },
            if (index & 0b001) == 0 { 1.0 } else { -1.0 },
        )
    }

    fn get_cubes_of_children(&self) -> Vec<SceneObject> {
        let mut out = vec![];
        for (i, child) in self.children.iter().enumerate() {
            let color = OctTreeNode::index_to_color(i);
            if child.children.len() != 0 {
                let cubes = child.get_cubes_of_children();
                for cube in cubes {
                    out.push(cube);
                }
            } else {
                let cube = SceneObject::new_cube(child.center, child.half_width * 2.0, color);
                out.push(cube);
            }
        }
        return out;
    }

    fn get_lines_of_children(&self, only_clicks: bool) -> Vec<Line> {
        let mut out = vec![];

        // log!("in octtree only clicks is {:?}", only_clicks);
        if only_clicks && !self.touched {
            // log!("returning early because only clicks and not touched");
            return out;
        }

        for (i, child) in self.children.iter().enumerate() {
            let color = OctTreeNode::index_to_color(i);
            if child.children.len() != 0 {
                let lines = child.get_lines_of_children(only_clicks);
                for line in lines {
                    out.push(line);
                }
            }

            if true {
                out.push(Line {
                    start: child.center + vec3(-1.0, 1.0, 1.0) * child.half_width,
                    end: child.center + vec3(1.0, 1.0, 1.0) * child.half_width,
                    color,
                });
                out.push(Line {
                    start: child.center + vec3(-1.0, 1.0, -1.0) * child.half_width,
                    end: child.center + vec3(1.0, 1.0, -1.0) * child.half_width,
                    color,
                });
                out.push(Line {
                    start: child.center + vec3(1.0, 1.0, -1.0) * child.half_width,
                    end: child.center + vec3(1.0, 1.0, 1.0) * child.half_width,
                    color,
                });
                out.push(Line {
                    start: child.center + vec3(-1.0, 1.0, -1.0) * child.half_width,
                    end: child.center + vec3(-1.0, 1.0, 1.0) * child.half_width,
                    color,
                });

                out.push(Line {
                    start: child.center + vec3(-1.0, -1.0, 1.0) * child.half_width,
                    end: child.center + vec3(1.0, -1.0, 1.0) * child.half_width,
                    color,
                });
                out.push(Line {
                    start: child.center + vec3(-1.0, -1.0, -1.0) * child.half_width,
                    end: child.center + vec3(1.0, -1.0, -1.0) * child.half_width,
                    color,
                });
                out.push(Line {
                    start: child.center + vec3(1.0, -1.0, -1.0) * child.half_width,
                    end: child.center + vec3(1.0, -1.0, 1.0) * child.half_width,
                    color,
                });
                out.push(Line {
                    start: child.center + vec3(-1.0, -1.0, -1.0) * child.half_width,
                    end: child.center + vec3(-1.0, -1.0, 1.0) * child.half_width,
                    color,
                });

                out.push(Line {
                    start: child.center + vec3(-1.0, -1.0, 1.0) * child.half_width,
                    end: child.center + vec3(-1.0, 1.0, 1.0) * child.half_width,
                    color,
                });
                out.push(Line {
                    start: child.center + vec3(1.0, -1.0, 1.0) * child.half_width,
                    end: child.center + vec3(1.0, 1.0, 1.0) * child.half_width,
                    color,
                });

                out.push(Line {
                    start: child.center + vec3(1.0, -1.0, -1.0) * child.half_width,
                    end: child.center + vec3(1.0, 1.0, -1.0) * child.half_width,
                    color,
                });
                out.push(Line {
                    start: child.center + vec3(-1.0, -1.0, -1.0) * child.half_width,
                    end: child.center + vec3(-1.0, 1.0, -1.0) * child.half_width,
                    color,
                });
            }
        }
        return out;
    }

    fn propogate_splats_to_children(&mut self, depth: usize, params: SplitParams, all_splats: &[Splat]) {
        let len = self.splat_indices.len();
        if len < params.split_limit || depth >= params.max_depth {
            return;
        }

        assert!(self.children.is_empty(), "octreenode already has children!");

        // Pre-allocate children with empty index vectors
        self.children.reserve_exact(8);
        let mut child_indices: [Vec<usize>; 8] = Default::default();
        
        // Distribute indices to children based on splat positions
        for &idx in &self.splat_indices {
            let splat = &all_splats[idx];
            if splat.opacity < 0.02 {
                continue;
            }

            let child_idx = (
                if splat.x > self.center.x { 0 } else { 4 }
            ) + (
                if splat.y > self.center.y { 0 } else { 2 }
            ) + (
                if splat.z > self.center.z { 0 } else { 1 }
            );
            
            child_indices[child_idx].push(idx);
        }

        // Create children only if they have splats
        for i in 0..8 {
            let direction = OctTreeNode::index_to_direction(i);
            let new_center = self.center + direction * self.half_width / 2.0;
            let child = OctTreeNode::new(
                std::mem::take(&mut child_indices[i]),
                new_center,
                self.half_width / 2.0,
                params,
                all_splats
            );
            self.children.push(child);
        }
        
        // Clear parent indices to save memory
        self.splat_indices.clear();
        self.splat_indices.shrink_to_fit();
    }

    pub fn find_splats_in_radius(&mut self, center: Vec3, radius: f32, all_splats: &[Splat]) -> Vec<OctTreeSplat> {
        let mut out = Vec::new();
        
        // Early exit if sphere doesn't intersect this node's bounds
        if center.x - radius > self.center.x + self.half_width
            || center.x + radius < self.center.x - self.half_width
            || center.y - radius > self.center.y + self.half_width
            || center.y + radius < self.center.y - self.half_width
            || center.z - radius > self.center.z + self.half_width
            || center.z + radius < self.center.z - self.half_width
        {
            self.touched = false;
            return out;
        }
        self.touched = true;

        if self.children.is_empty() {
            // Leaf node - check splats directly
            out.reserve(self.splat_indices.len());
            for &idx in &self.splat_indices {
                let splat = &all_splats[idx];
                if glm::distance(&vec3(splat.x, splat.y, splat.z), &center) <= radius {
                    out.push(OctTreeSplat {
                        x: splat.x,
                        y: splat.y,
                        z: splat.z,
                        opacity: splat.opacity,
                        index: idx,
                    });
                }
            }
        } else {
            // Internal node - recurse to children
            for child in &mut self.children {
                let child_splats = child.find_splats_in_radius(center, radius, all_splats);
                out.extend(child_splats);
            }
        }

        out
    }
}

impl OctTree {
    pub fn new(splats: Vec<Splat>) -> Self {
        log!("new octtree with {} splats", splats.len());

        // Choose adaptive parameters based on total splat count.
        let total = splats.len();
        let params = if total > 1_000_000 {
            // Very large cloud – keep the tree shallow, split less aggressively.
            log!("using very large cloud params");
            SplitParams {
                split_limit: 50_000,
                max_depth: 5,
            }
        } else if total > 300_000 {
            // Medium-large cloud
            log!("using medium-large cloud params");
            SplitParams {
                split_limit: 20_000,
                max_depth: 6,
            }
        } else {
            // Default parameters for small/medium scenes
            SplitParams {
                split_limit: DEFAULT_SPLIT_LIMIT,
                max_depth: DEFAULT_MAX_DEPTH,
            }
        };

        let (root, all_splats) = OctTreeNode::new_root(splats, vec3(0.0, 0.0, 0.0), 10.0, params);
        OctTree { root, all_splats }
    }

    pub fn get_splat(&self, index: usize) -> Option<&Splat> {
        self.all_splats.get(index)
    }

    pub fn get_all_splats(&self) -> &[Splat] {
        &self.all_splats
    }

    pub fn get_cubes(&self) -> Vec<SceneObject> {
        return self.root.get_cubes_of_children();
    }

    pub fn get_lines(&self, only_clicks: bool) -> Vec<Line> {
        return self.root.get_lines_of_children(only_clicks);
    }

    pub fn find_splats_in_radius(&mut self, center: Vec3, radius: f32) -> Vec<OctTreeSplat> {
        log!("finding splats in radius {:?}", center);
        self.root.find_splats_in_radius(center, radius, &self.all_splats)
    }

    pub fn get_root(&self) -> &OctTreeNode {
        &self.root
    }

    pub fn get_root_mut(&mut self) -> &mut OctTreeNode {
        &mut self.root
    }
}
