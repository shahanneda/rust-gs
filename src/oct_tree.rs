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

#[derive(Debug)]
pub struct OctTreeNode {
    pub children: Vec<OctTreeNode>,
    pub splats: Vec<OctTreeSplat>,
    pub center: Vec3,
    pub half_width: f32,
    pub touched: bool,
}

#[derive(Debug)]
pub struct OctTree {
    pub root: OctTreeNode,
}
// mapping from i to top right back, top right front, bottom right back, bottom right front, top left back, top left front, bottom left back, bottom left front
// const SPLIT_LIMIT: usize = 10;
const SPLIT_LIMIT: usize = 1000;
const MAX_DEPTH: usize = 10;

impl OctTreeNode {
    pub fn new(splats: Vec<Splat>, center: Vec3, half_width: f32) -> Self {
        let len = splats.len();
        let mut oct_tree_splats = vec![];
        for (i, splat) in splats.iter().enumerate() {
            oct_tree_splats.push(OctTreeSplat {
                x: splat.x,
                y: splat.y,
                z: splat.z,
                opacity: splat.opacity,
                index: i,
            });
        }

        let mut out = OctTreeNode {
            children: vec![],
            splats: oct_tree_splats,
            center,
            half_width,
            touched: false,
        };

        out.propogate_splats_to_children(0);

        return out;
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

        log!("in octtree only clicks is {:?}", only_clicks);
        if only_clicks && !self.touched {
            log!("returning early because only clicks and not touched");
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

    fn propogate_splats_to_children(&mut self, depth: usize) {
        let len = self.splats.len();
        if len < SPLIT_LIMIT {
            return;
        }
        if depth >= MAX_DEPTH {
            return;
        }

        assert!(self.children.len() == 0, "octreenode already has children!");

        for i in 0..8 {
            let direction = OctTreeNode::index_to_direction(i);
            let new_center = self.center + direction * self.half_width / 2.0;

            let child = OctTreeNode::new(vec![], new_center, self.half_width / 2.0);
            self.children.push(child);
        }

        for splat in &self.splats {
            if splat.opacity < 0.02 {
                continue;
            }

            if splat.x > self.center.x {
                if splat.y > self.center.y {
                    if splat.z > self.center.z {
                        // top right back
                        self.children[0].splats.push(splat.clone());
                    } else {
                        // top right front
                        self.children[1].splats.push(splat.clone());
                    }
                } else {
                    if splat.z > self.center.z {
                        // bottom right back
                        self.children[2].splats.push(splat.clone());
                    } else {
                        // bottom right front
                        self.children[3].splats.push(splat.clone());
                    }
                }
            } else {
                if splat.y > self.center.y {
                    if splat.z > self.center.z {
                        // top left back
                        self.children[4].splats.push(splat.clone());
                    } else {
                        // top left front
                        self.children[5].splats.push(splat.clone());
                    }
                } else {
                    if splat.z > self.center.z {
                        // bottom left back
                        self.children[6].splats.push(splat.clone());
                    } else {
                        // bottom left front
                        self.children[7].splats.push(splat.clone());
                    }
                }
            }
        }

        for child in &mut self.children {
            log!("child has {} splats", child.splats.len());
            child.propogate_splats_to_children(depth + 1);
        }
    }

    pub fn find_splats_in_radius(&mut self, center: Vec3, radius: f32) -> Vec<OctTreeSplat> {
        let mut out = vec![];
        log!("finding splats in radius {:?}", center);
        if center.x - radius > self.center.x + self.half_width
            || center.x + radius < self.center.x - self.half_width
            || center.y - radius > self.center.y + self.half_width
            || center.y + radius < self.center.y - self.half_width
            || center.z - radius > self.center.z + self.half_width
            || center.z + radius < self.center.z - self.half_width
        {
            log!(
                "returning early x: {}, max x: {}, min x: {}",
                center.x,
                self.center.x + self.half_width,
                self.center.x - self.half_width
            );
            self.touched = false;
            return out;
        }
        self.touched = true;

        if self.children.len() == 0 {
            log!(
                "have no children checking splats in child center: {:?}, request center: {:?} request radius: {}",
                self.center, center, radius
                );
            log!("looping through {} splats", self.splats.len());
            for splat in &self.splats {
                // out.push(splat.clone());
                if glm::distance(&vec3(splat.x, splat.y, splat.z), &center) <= radius {
                    //     log!("found splat {:?}", splat);
                    out.push(splat.clone());
                }
            }
        } else {
            for child in &mut self.children {
                let child_splats = child.find_splats_in_radius(center, radius);
                out.extend(child_splats);
            }
        }

        return out;
    }
}

impl OctTree {
    pub fn new(splats: Vec<Splat>) -> Self {
        // let root = OctTreeNode::new(splats);
        let root = OctTreeNode::new(splats, vec3(0.0, 0.0, 0.0), 10.0);
        return OctTree { root: root };
    }

    pub fn get_cubes(&self) -> Vec<SceneObject> {
        return self.root.get_cubes_of_children();
    }

    pub fn get_lines(&self, only_clicks: bool) -> Vec<Line> {
        return self.root.get_lines_of_children(only_clicks);
    }

    pub fn find_splats_in_radius(&mut self, center: Vec3, radius: f32) -> Vec<OctTreeSplat> {
        log!("finding splats in radius {:?}", center);
        return self.root.find_splats_in_radius(center, radius);
    }
}
