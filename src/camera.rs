use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashSet;
use std::rc::Rc;

use glm::vec2;
use glm::vec3;
use glm::Mat4;
use glm::Vec2;
use glm::Vec3;
use nalgebra_glm::vec4;
use crate::log;
extern crate eframe;
extern crate js_sys;
extern crate nalgebra_glm as glm;
extern crate ply_rs;
extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;
use web_sys::MouseEvent;

const MOVE_SPEED: f32 = 0.05;
pub struct Camera {
    pub pos: Vec3,
    pub rot: Vec2,
    pub is_dragging: bool,
    pub last_mouse_pos: Vec2,
}

impl Camera {
    pub fn new(pos: Vec3, rot: Vec2) -> Self {
        Self {
            pos: vec3(0.0, 0.0, 0.0),
            rot: vec2(0.0, 0.0),
            is_dragging: false,
            last_mouse_pos: vec2(0.0, 0.0),
        }
    }

    pub fn setup_mouse_events(
        camera: &Rc<RefCell<Camera>>,
        canvas: &web_sys::HtmlCanvasElement,
        document: &web_sys::Document,
    ) -> Result<(), JsValue> {
        // Mouse down handler
        let cam_mousedown = camera.clone();
        let mousedown_cb = Closure::wrap(Box::new(move |e: MouseEvent| {
            let mut camera = cam_mousedown.as_ref().borrow_mut();
            camera.is_dragging = true;
            camera.last_mouse_pos = vec2(e.client_x() as f32, e.client_y() as f32);
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("mousedown", mousedown_cb.as_ref().unchecked_ref())?;
        mousedown_cb.forget();

        let cam_mousemove = camera.clone();
        let mousemove_cb = Closure::wrap(Box::new(move |e: MouseEvent| {
            let mut cam = cam_mousemove.as_ref().borrow_mut();
            if cam.is_dragging {
                let current_pos = Vec2::new(e.client_x() as f32, e.client_y() as f32);
                let delta = current_pos - cam.last_mouse_pos;

                // Adjust these factors to control rotation speed
                let rotation_factor_x = 0.001;
                let rotation_factor_y = 0.001;

                cam.rot.y -= -delta.x * rotation_factor_x;
                cam.rot.x -= -delta.y * rotation_factor_y;

                // Clamp vertical rotation to avoid flipping
                cam.rot.x = cam
                    .rot
                    .x
                    .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
                cam.last_mouse_pos = current_pos;
            }
        }) as Box<dyn FnMut(_)>);
        document
            .add_event_listener_with_callback("mousemove", mousemove_cb.as_ref().unchecked_ref())?;
        mousemove_cb.forget();

        let cam_mouseup = camera.clone();
        let mouseup_cb = Closure::wrap(Box::new(move |_: MouseEvent| {
            let mut cam = cam_mouseup.as_ref().borrow_mut();
            cam.is_dragging = false;
        }) as Box<dyn FnMut(_)>);
        document
            .add_event_listener_with_callback("mouseup", mouseup_cb.as_ref().unchecked_ref())?;
        mouseup_cb.forget();
        Ok(())
    }

//     fn clone(&self) -> Self {
//         Self {
//             pos: self.pos.clone(),
//             rot: self.rot.clone(),
//             is_dragging: self.is_dragging,
//             last_mouse_pos: self.last_mouse_pos.clone(),
//         }
//     }

    pub fn update_translation_from_keys(self: &mut Camera, keys_pressed: &RefCell<HashSet<String>>) {
        let keys = keys_pressed.borrow();
        let mut cam_translation_local = vec3(0.0, 0.0, 0.0);
        if keys.contains("w") {
            cam_translation_local.z -= MOVE_SPEED;
        }
        if keys.contains("s") {
            cam_translation_local.z += MOVE_SPEED;
        }
        if keys.contains("a") {
            cam_translation_local.x += MOVE_SPEED;
        }
        if keys.contains("d") {
            cam_translation_local.x -= MOVE_SPEED;
        }
        if keys.contains(" ") {
            cam_translation_local.y -= MOVE_SPEED;
        }
        if keys.contains("Shift") {
            cam_translation_local.y += MOVE_SPEED;
        }

        if cam_translation_local != vec3(0.0, 0.0, 0.0) {
            let cam_to_world = self.get_camera_to_world_matrix();
            let cam_pos_after_moving = cam_to_world * vec4(cam_translation_local.x, cam_translation_local.y, cam_translation_local.z, 0.0);
            self.pos += vec3(cam_pos_after_moving.x, cam_pos_after_moving.y, cam_pos_after_moving.z);
        }

        if keys.contains("ArrowUp") {
            self.rot.x -= 0.1;
        }
        if keys.contains("ArrowDown") {
            self.rot.x += 0.1;
        }
        if keys.contains("ArrowLeft") {
            self.rot.y -= 0.1;
        }
        if keys.contains("ArrowRight") {
            self.rot.y += 0.1;
        }
    }

    pub fn get_world_to_camera_matrix(self: &Camera) -> Mat4 {
        let mut camera: Mat4 = glm::identity();
        camera = glm::rotate_z(&camera, 1.0 * glm::pi::<f32>());
        camera = glm::rotate_x(&camera, self.rot.x);
        camera = glm::rotate_y(&camera, self.rot.y);
        camera = glm::translate(&camera, &self.pos);
        return camera;
    }

    pub fn get_camera_to_world_matrix(self: &Camera) -> Mat4 {
        return self.get_world_to_camera_matrix().try_inverse().unwrap();
    }
}
