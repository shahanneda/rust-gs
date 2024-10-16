use js_sys::Int32Array;
use js_sys::Uint32Array;
use nalgebra_glm::pi;
use serde::{Serialize, Deserialize};
use web_sys::Worker;
use web_sys::WorkerOptions;
use web_sys::WorkerType;
use std::cell::RefCell;
use std::convert::TryInto;
use std::num;
use std::ops::DerefMut;
// use std::num;
use std::rc::Rc;

use eframe::glow::TRIANGLES;
use eframe::glow::TRIANGLE_STRIP;
use egui::debug_text::print;
use egui::util::undoer::Settings;
// use egui::epaint::Vertex;
// use egui::frame;
use glm::log;
use glm::vec2;
use glm::vec3;
use glm::vec4;
use glm::vec4_to_vec3;
use glm::Mat4;
use glm::Vec2;
use glm::Vec3;
use crate::scene::Scene;  // Use crate:: to import from your lib.rs
use crate::loader::loader;  // If you need the loader
use crate::log;
use crate::timer::Timer;
use crate::utils::set_panic_hook;
// use crate::shader;
use crate::shader;
extern crate js_sys;
extern crate ply_rs;
extern crate wasm_bindgen;
extern crate eframe;
extern crate nalgebra_glm as glm;
use crate::scene_geo;
use js_sys::{Float32Array, WebAssembly};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;
use web_sys::Event;
use web_sys::HtmlCanvasElement;
use web_sys::HtmlInputElement;
use web_sys::WebGl2RenderingContext;
use web_sys::WebGlBuffer;
use web_sys::WebGlProgram;
use web_sys::WebGlVertexArrayObject;
use web_sys::MouseEvent;
use web_sys::WebGlTexture;
use web_sys::WebGlUniformLocation;


#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
    Third = 99,
}


#[wasm_bindgen]
pub struct GSBuffer {
    pub width: u8,
    pub height: u8,
    cells: Vec<u8>
}

#[wasm_bindgen]
impl GSBuffer {
    pub fn new() -> GSBuffer {
        let size = 10;
        let cells = vec![0, size];
        return GSBuffer {
            width: 10,
            height: 10,
            cells,
        };
    }

    pub fn k() {
    }

    pub fn display(&self) {
        let num = self.cells[0];
    }
}

#[wasm_bindgen]
extern "C" {
    // #[wasm_bindgen(js_namespace = console)]
    // fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    // #[wasm_bindgen(js_namespace = console, js_name = log)]
    // fn log_u32(a: u32);

    // // Multiple arguments too!
    // #[wasm_bindgen(js_namespace = console, js_name = log)]
    // fn log_many(a: &str, b: &str);

    #[wasm_bindgen(js_name = getWebGLContext)]
    fn getWebGLContext() -> WebGl2RenderingContext;

    #[wasm_bindgen(js_name = test)]
    fn test_js();

    #[wasm_bindgen(js_name = prompt)]
    fn promptJS(str : &str) -> String;

    #[wasm_bindgen(js_namespace = WebAssembly, js_name = Memory)]
    pub type Memory;

    #[wasm_bindgen(constructor)]
    fn new(initial_size: u32) -> Memory;
}

// fn bare_bones() {
//     log("testing ");
//     log_u32(42);
//     log_many("Logging", "many values!");
// }

pub static mut worker_initialized: bool = false;

#[wasm_bindgen]
pub fn testing(val: f32) {
    log!("testing: {}", val);
}

fn update_buffer_data(gl: &WebGl2RenderingContext, buffer: &WebGlBuffer, data: Float32Array) {
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &data,
        WebGl2RenderingContext::STATIC_DRAW,
    );
}

fn update_buffer_data_u32(gl: &WebGl2RenderingContext, buffer: &WebGlBuffer, data: Uint32Array) {
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &data,
        WebGl2RenderingContext::STATIC_DRAW,
    );
}

fn create_buffer(gl: &WebGl2RenderingContext) -> Result<WebGlBuffer, &'static str> {
    let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
    return Ok(buffer);
}


fn create_attribute_and_get_location(gl: &WebGl2RenderingContext, buffer: &WebGlBuffer, program: &WebGlProgram, name: &str, divisor: bool, size: i32, type_: u32) -> u32{
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
    let coord = gl.get_attrib_location(&program, name) as u32;
    gl.enable_vertex_attrib_array(coord);

    if type_ == WebGl2RenderingContext::UNSIGNED_INT {
        gl.vertex_attrib_i_pointer_with_i32(coord, size, type_, 0, 0);
    } else if type_ == WebGl2RenderingContext::FLOAT {
        // Data is converted to float in the shader
        // the type referes to the type of the data in the buffer, not the type of the data in the shader
        // https://stackoverflow.com/questions/78203199/webgl-2-0-unsigned-integer-input-variable#answer-78203412
        gl.vertex_attrib_pointer_with_i32(coord, size, type_, false, 0, 0);
    }else{
        panic!("Invalid type for attribute");
    }
    if divisor {
        gl.vertex_attrib_divisor(coord, 1);
    }
    return coord;
}


fn float32_array_from_vec(vec: &[f32]) -> Float32Array {
    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>().unwrap()
        .buffer();
    let location: u32 = vec.as_ptr() as u32 / 4;
    return Float32Array::new(&memory_buffer).subarray(location, location + vec.len() as u32);
}

fn uint32_array_from_vec(vec: &[u32]) -> Uint32Array {
    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>().unwrap()
        .buffer();
    let location: u32 = vec.as_ptr() as u32 / 4;
    return Uint32Array::new(&memory_buffer).subarray(location, location + vec.len() as u32);
}

struct WebGLSetupResult {
    gl: WebGl2RenderingContext,
    splat_shader: WebGlProgram,
    splat_vao: WebGlVertexArrayObject,
    // vertex_buffer: WebGlBuffer,
    // color_buffer: WebGlBuffer,
    // position_offset_buffer: WebGlBuffer,
    // cov3da_buffer: WebGlBuffer,
    // cov3db_buffer: WebGlBuffer,
    // opacity_buffer: WebGlBuffer,
    geo_shader: WebGlProgram,
    geo_vertex_buffer: WebGlBuffer,
    splat_index_buffer: WebGlBuffer,
    geo_count: i32,
    geo_vao: WebGlVertexArrayObject,
    color_texture: WebGlTexture,
    position_texture: WebGlTexture,
    cov3da_texture: WebGlTexture,
    cov3db_texture: WebGlTexture,
    opacity_texture: WebGlTexture,
}

fn create_texture(gl: &WebGl2RenderingContext, program: &WebGlProgram, name : &str, active_texture: u32) -> Result<(WebGlTexture, WebGlUniformLocation), JsValue> {
    let texture = gl.create_texture().ok_or("Failed to create texture")?;
    gl.active_texture(active_texture);
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

    let empty_array = Float32Array::new_with_length(0);
    put_data_into_texture(&gl, &texture, &empty_array)?;
    // let level = 0;
    // let internal_format = WebGl2RenderingContext::RGB32F as i32;
    // let width = 1;
    // let height = number_of_items;
    // let border = 0;
    // let format = WebGl2RenderingContext::RGB;
    // let type_ = WebGl2RenderingContext::FLOAT;
    // let data = [
    //     // 0.1f32, 1.0, 0.6, // greenish
    //     // 0.8f32, 0.1, 0.6, // meganta
    // ]; 

    // // Convert f32 array to Uint8Array
    // // Because we don't actually have rust bindings for Float32Arrays in the webgl crate, we do this to directly pass a JS array to the texture
    // let data_array = unsafe { js_sys::Float32Array::view(&data) };

    // gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
    //     WebGl2RenderingContext::TEXTURE_2D,
    //     level,
    //     internal_format,
    //     width,
    //     height,
    //     border,
    //     format,
    //     type_,
    //     Some(&data_array),
    // )?;

    gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_MIN_FILTER, WebGl2RenderingContext::NEAREST as i32);
    gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_MAG_FILTER, WebGl2RenderingContext::NEAREST as i32);
    gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_WRAP_S, WebGl2RenderingContext::CLAMP_TO_EDGE as i32);
    gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_WRAP_T, WebGl2RenderingContext::CLAMP_TO_EDGE as i32);

    let location = gl.get_uniform_location(program, name)
        .ok_or("Failed to get uniform location")?;

    Ok((texture, location))
}

fn update_webgl_buffers(scene: &Scene, webgl: &WebGLSetupResult){
    let _timer = Timer::new("update_webgl_buffers");
    // let mut splat_centers = Vec::new();
    // let mut splat_colors = Vec::new();
    // let mut splat_cov3da = Vec::new();
    // let mut splat_cov3db = Vec::new();
    // let mut splat_opacities = Vec::new();
    // let mut splat_indices = Vec::new();


    // for s in &scene.splats {
    // //     // splat_centers.extend_from_slice(&[s.x, s.y, s.z]);
    // //     // splat_colors.extend_from_slice(&[s.r, s.g, s.b]);
    // //     // splat_cov3da.extend_from_slice(&[s.cov3d[0], s.cov3d[1], s.cov3d[2]]);
    // //     // splat_cov3db.extend_from_slice(&[s.cov3d[3], s.cov3d[4], s.cov3d[5]]);
    // //     // splat_opacities.push(s.opacity);
    //     splat_indices.push(s.index);
    // }

    // webgl.gl.use_program(Some(&webgl.splat_shader));
    // webgl.gl.bind_vertex_array(Some(&webgl.splat_vao));
    // update_buffer_data_u32(&webgl.gl, &webgl.splat_index_buffer, int32_array_from_vec(&splat_indices));

    // update_buffer_data(&webgl.gl, &webgl.color_buffer, float32_array_from_vec(&splat_colors));
    // update_buffer_data(&webgl.gl, &webgl.position_offset_buffer, float32_array_from_vec(&splat_centers));
    // update_buffer_data(&webgl.gl, &webgl.cov3da_buffer, float32_array_from_vec(&splat_cov3da));
    // update_buffer_data(&webgl.gl, &webgl.cov3db_buffer, float32_array_from_vec(&splat_cov3db));
    // update_buffer_data(&webgl.gl, &webgl.opacity_buffer, float32_array_from_vec(&splat_opacities));
}


fn update_splat_indices(scene: &Scene, webgl: &WebGLSetupResult, splat_indices: &Vec<u32>){
    // print out first few
    log!("updating splat_indices: {:?}", &splat_indices[0..3]);
    let _timer = Timer::new("update_splat_indices");
    webgl.gl.use_program(Some(&webgl.splat_shader));
    webgl.gl.bind_vertex_array(Some(&webgl.splat_vao));
    update_buffer_data_u32(&webgl.gl, &webgl.splat_index_buffer, uint32_array_from_vec(&splat_indices));
}

fn update_webgl_textures(scene: &Scene, webgl: &WebGLSetupResult) -> Result<(), JsValue>{
    let mut splat_positions = Vec::new();
    let mut splat_colors = Vec::new();
    let mut splat_cov3da = Vec::new();
    let mut splat_cov3db = Vec::new();
    let mut splat_opacities = Vec::new();

    for s in &scene.splats {
        splat_positions.extend_from_slice(&[s.x, s.y, s.z]);
        splat_colors.extend_from_slice(&[s.r, s.g, s.b]);
        splat_cov3da.extend_from_slice(&[s.cov3d[0], s.cov3d[1], s.cov3d[2]]);
        splat_cov3db.extend_from_slice(&[s.cov3d[3], s.cov3d[4], s.cov3d[5]]);
        splat_opacities.extend_from_slice(&[s.opacity, 0.0, 0.0]);
    }

    // log!("splat_colors: {:?}", &splat_colors[0..3]);
    webgl.gl.active_texture(WebGl2RenderingContext::TEXTURE0 + COLOR_TEXTURE_UNIT);
    put_data_into_texture(&webgl.gl, &webgl.color_texture, &float32_array_from_vec(&splat_colors))?;

    webgl.gl.active_texture(WebGl2RenderingContext::TEXTURE0 + POSITION_TEXTURE_UNIT);
    put_data_into_texture(&webgl.gl, &webgl.position_texture, &float32_array_from_vec(&splat_positions))?;

    webgl.gl.active_texture(WebGl2RenderingContext::TEXTURE0 + COV3DA_TEXTURE_UNIT);
    put_data_into_texture(&webgl.gl, &webgl.cov3da_texture, &float32_array_from_vec(&splat_cov3da))?;

    webgl.gl.active_texture(WebGl2RenderingContext::TEXTURE0 + COV3DB_TEXTURE_UNIT);
    put_data_into_texture(&webgl.gl, &webgl.cov3db_texture, &float32_array_from_vec(&splat_cov3db))?;

    webgl.gl.active_texture(WebGl2RenderingContext::TEXTURE0 + OPACITY_TEXTURE_UNIT);
    put_data_into_texture(&webgl.gl, &webgl.opacity_texture, &float32_array_from_vec(&splat_opacities))?;

    // put_data_into_texture(&webgl.gl, &webgl.color_texture, &float32_array_from_vec(&[
    //     1.0, 0.0, 0.0,
    //     0.0, 1.0, 0.0,
    //     ]))?;
    Ok(())
}

const TEXTURE_WIDTH: i32 = 2000;

fn put_data_into_texture(gl: &WebGl2RenderingContext, texture: &WebGlTexture, data_array: &Float32Array) -> Result<(), JsValue>{
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));

    let level = 0;
    let internal_format = WebGl2RenderingContext::RGB32F as i32;
    let width = TEXTURE_WIDTH;
    let number_of_values = data_array.length() as i32;
    // We add Texture_width -1 so that we always do a ceiling division
    let height = (number_of_values + TEXTURE_WIDTH - 1) / TEXTURE_WIDTH;  // Assuming 3 components (RGB) per pixel
    log!("height: {}", height);
    log!("length: {}", data_array.length());
    
    // resize data array to match the texture size
    // TODO: don't duplicat the array here, make sure arrays are the right size before passing into here
    let resized_data_array = Float32Array::new_with_length((TEXTURE_WIDTH * height * 3).try_into().unwrap());
    for i in 0..number_of_values {
        resized_data_array.set_index(i as u32, data_array.get_index(i as u32));
    }

    let border = 0;
    let format = WebGl2RenderingContext::RGB;
    let type_ = WebGl2RenderingContext::FLOAT;

    // Convert f32 array to Uint8Array
    // Because we don't actually have rust bindings for Float32Arrays in the webgl crate, we do this to directly pass a JS array to the texture
    // let data_array = unsafe { js_sys::Float32Array::view(data) };

    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
        WebGl2RenderingContext::TEXTURE_2D,
        level,
        internal_format,
        width,
        height,
        border,
        format,
        type_,
        Some(&resized_data_array),
    )?;
    Ok(())
}

const COLOR_TEXTURE_UNIT: u32 = 0;
const POSITION_TEXTURE_UNIT: u32 = 1;
const COV3DA_TEXTURE_UNIT: u32 = 2;
const COV3DB_TEXTURE_UNIT: u32 = 3;
const OPACITY_TEXTURE_UNIT: u32 = 4;



fn setup_webgl(gl: WebGl2RenderingContext, scene : &Scene) -> Result<WebGLSetupResult, JsValue> {
    let vertices: [f32; 3*4] = [
        -1.0, -1.0, 0.0, //
        1.0, -1.0, 0.0, //
        -1.0, 1.0, 0.0, //
        1.0, 1.0, 0.0, //
    ];
    // let vertices: [f32; 3*4] = [
    //     0.5, 0.5, 0.0, //
    //     -0.5, 0.5, 0.0, //
    //     -0.5, -0.5, 0.0, //
    //     0.5, -0.5, 0.0, //
    // ];
    // let vertices: [f32; 3*4] = [
    //     -0.0, 0.0, 0.0, //
    //     1.0, 0.0, 0.0, //
    //     -0.0, 1.0, 0.0, //
    //     1.0, 1.0, 0.0, //
    // ];
    // let vertices: [f32; 3*4] = [
    //     -0.1, -0.1, 0.0, //
    //     0.1, -0.1, 0.0, //
    //     -0.1, 0.1, 0.0, //
    //     0.1, 0.1, 0.0, //
    // ];
    let vertices = vertices.map(|v| v);

    let splat_vao = gl.create_vertex_array().unwrap();
    gl.bind_vertex_array(Some(&splat_vao));
    // let vertex_buffer = create_buffer(&gl).unwrap();
    // let color_buffer = create_buffer(&gl).unwrap();
    // let position_offset_buffer = create_buffer(&gl).unwrap();
    // let cov3da_buffer = create_buffer(&gl).unwrap();
    // let cov3db_buffer = create_buffer(&gl).unwrap();
    // let opacity_buffer = create_buffer(&gl).unwrap();
    let geo_vertex_buffer = create_buffer(&gl).unwrap();
    let splat_index_buffer = create_buffer(&gl).unwrap();

    // update_buffer_data(&gl, &vertex_buffer, float32_array_from_vec(&vertices));


    let geo_vao = gl.create_vertex_array().unwrap();
    gl.bind_vertex_array(Some(&geo_vao));
    let vertices = scene_geo::VERTICES.map(|v| v);
    update_buffer_data(&gl, &geo_vertex_buffer, float32_array_from_vec(&vertices));

    let splat_shader = shader::shader::create_splat_shader_program(&gl).unwrap();

    let geo_shader = shader::shader::create_geo_shader_program(&gl).unwrap();


    gl.active_texture(WebGl2RenderingContext::TEXTURE0);
    let (color_texture, color_texture_location) = create_texture(&gl, &splat_shader, "u_color_texture", WebGl2RenderingContext::TEXTURE0 + COLOR_TEXTURE_UNIT)?;

    gl.active_texture(WebGl2RenderingContext::TEXTURE0 + POSITION_TEXTURE_UNIT);
    let (position_texture, position_texture_location) = create_texture(&gl, &splat_shader, "u_position_texture", WebGl2RenderingContext::TEXTURE0 + POSITION_TEXTURE_UNIT)?;
    let (cov3da_texture, cov3da_texture_location) = create_texture(&gl, &splat_shader, "u_cov3da_texture", WebGl2RenderingContext::TEXTURE0 + COV3DA_TEXTURE_UNIT)?;
    let (cov3db_texture, cov3db_texture_location) = create_texture(&gl, &splat_shader, "u_cov3db_texture", WebGl2RenderingContext::TEXTURE0 + COV3DB_TEXTURE_UNIT)?;
    let (opacity_texture, opacity_texture_location) = create_texture(&gl, &splat_shader, "u_opacity_texture", WebGl2RenderingContext::TEXTURE0 + OPACITY_TEXTURE_UNIT)?;

    let result  = WebGLSetupResult{
        gl: gl,
        splat_shader,
        // vertex_buffer,
        // color_buffer,
        // position_offset_buffer,
        // cov3da_buffer,
        // cov3db_buffer,
        // opacity_buffer,
        splat_vao,
        geo_shader,
        geo_vertex_buffer,
        splat_index_buffer,
        geo_count: vertices.len() as i32,
        geo_vao,
        color_texture,
        position_texture,
        cov3da_texture,
        cov3db_texture,
        opacity_texture,
    };

    result.gl.use_program(Some(&result.splat_shader));
    result.gl.bind_vertex_array(Some(&result.splat_vao));

    update_webgl_buffers(scene, &result);
    update_webgl_textures(scene, &result).expect("failed to update webgl textures for the first time!");

    // create_attribute_and_get_location(&result.gl, &result.vertex_buffer, &result.splat_shader, "v_pos", false, 3, WebGl2RenderingContext::FLOAT);
    // create_attribute_and_get_location(&result.gl, &result.color_buffer, &result.splat_shader, "s_color", true, 3, WebGl2RenderingContext::FLOAT);
    // create_attribute_and_get_location(&result.gl, &result.position_offset_buffer, &result.splat_shader, "s_center", true, 3, WebGl2RenderingContext::FLOAT);
    // create_attribute_and_get_location(&result.gl, &result.cov3da_buffer, &result.splat_shader, "s_cov3da", true, 3, WebGl2RenderingContext::FLOAT);
    // create_attribute_and_get_location(&result.gl, &result.cov3db_buffer, &result.splat_shader, "s_cov3db", true, 3, WebGl2RenderingContext::FLOAT);
    // create_attribute_and_get_location(&result.gl, &result.opacity_buffer, &result.splat_shader, "s_opacity", true, 1, WebGl2RenderingContext::FLOAT);
    create_attribute_and_get_location(&result.gl, &result.splat_index_buffer, &result.splat_shader, "s_index", true, 1, WebGl2RenderingContext::UNSIGNED_INT);

    result.gl.pixel_storei(WebGl2RenderingContext::UNPACK_ALIGNMENT, 1);
    result.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
    result.gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&result.color_texture));
    set_texture_uniform_value(&result.splat_shader, &result.gl, "u_color_texture", &result.color_texture, COLOR_TEXTURE_UNIT);

    result.gl.pixel_storei(WebGl2RenderingContext::UNPACK_ALIGNMENT, 1);
    result.gl.active_texture(WebGl2RenderingContext::TEXTURE1);
    result.gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&result.position_texture));
    set_texture_uniform_value(&result.splat_shader, &result.gl, "u_position_texture", &result.position_texture, POSITION_TEXTURE_UNIT);

    result.gl.active_texture(WebGl2RenderingContext::TEXTURE0 + COV3DA_TEXTURE_UNIT);
    result.gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&result.cov3da_texture));
    set_texture_uniform_value(&result.splat_shader, &result.gl, "u_cov3da_texture", &result.cov3da_texture, COV3DA_TEXTURE_UNIT);

    result.gl.active_texture(WebGl2RenderingContext::TEXTURE0 + COV3DB_TEXTURE_UNIT);
    result.gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&result.cov3db_texture));
    set_texture_uniform_value(&result.splat_shader, &result.gl, "u_cov3db_texture", &result.cov3db_texture, COV3DB_TEXTURE_UNIT);

    result.gl.active_texture(WebGl2RenderingContext::TEXTURE0 + OPACITY_TEXTURE_UNIT);
    result.gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&result.opacity_texture));
    set_texture_uniform_value(&result.splat_shader, &result.gl, "u_opacity_texture", &result.opacity_texture, OPACITY_TEXTURE_UNIT);
    

    // geo shader
    result.gl.use_program(Some(&result.geo_shader));
    result.gl.bind_vertex_array(Some(&result.geo_vao));
    create_attribute_and_get_location(&result.gl, &result.geo_vertex_buffer, &result.geo_shader, "v_pos", false, 3, WebGl2RenderingContext::FLOAT);

    return Ok(result);
}


fn set_float_uniform_value(shader_program: &WebGlProgram, gl: &WebGl2RenderingContext, name: &str, value: f32){ 
    // log!("name: {}", name);
    let uniform_location = gl.get_uniform_location(&shader_program, name).unwrap();
    gl.uniform1f(Some(&uniform_location), value);
}

fn set_vec3_uniform_value(shader_program: &WebGlProgram, gl: &WebGl2RenderingContext, name: &str, value: [f32; 3]){ 
    // log!("name: {}", name);
    let uniform_location = gl.get_uniform_location(&shader_program, name).unwrap();
    gl.uniform3fv_with_f32_array(Some(&uniform_location), value.as_slice());
}

fn set_texture_uniform_value(shader_program: &WebGlProgram, gl: &WebGl2RenderingContext, name: &str, texture: &WebGlTexture, active_texture: u32){
    let uniform_location = gl.get_uniform_location(&shader_program, name).unwrap();
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
    gl.uniform1i(Some(&uniform_location), active_texture as i32);
}

// const invertRow = (mat, row) => {
//   mat[row + 0] = -mat[row + 0];
//   mat[row + 4] = -mat[row + 4];
//   mat[row + 8] = -mat[row + 8];
//   mat[row + 12] = -mat[row + 12];
// };
fn invertRow(mat: &mut glm::Mat4, row: usize){
  mat[row + 0] = -mat[row + 0];
  mat[row + 4] = -mat[row + 4];
  mat[row + 8] = -mat[row + 8];
  mat[row + 12] = -mat[row + 12];
}


fn draw(webgl: &WebGLSetupResult, canvas: &web_sys::HtmlCanvasElement, frame_num: i32, num_vertices: i32, vpm: glm::Mat4, vm: glm::Mat4){
    let gl = &webgl.gl;
    let width = canvas.width() as i32;
    let height = canvas.height() as i32;
    // let mut model: Mat4 = glm::identity();
    // let model_scale = 3.0f32;
    // model = glm::translate(&model, &glm::vec3(0.0f32, 0.0f32, 0.0f32));
    // // model = glm::rotate_y(&model, current_amount*2.0*glm::pi::<f32>());
    // model = glm::scale(&model, &glm::vec3(model_scale, model_scale, model_scale));
    // camera = glm::translate(&camera, &glm::vec3(0f32, 0f32, 0f32));
    // let mut proj = glm::ortho(0f32, 800f32, 0f32, 1000f32, 0.0f32, 10f32);
    // glm::mat4 proj = glm::perspective(glm::radians(45.0f), (float)width/(float)height, 0.1f, 100.0f);
    // proj.fill_with_identity();


    // let model_uniform_location = gl.get_uniform_location(&shader_program, "model").unwrap();
    // gl.uniform_matrix4fv_with_f32_array(Some(&model_uniform_location), false, model.as_slice());
    gl.use_program(Some(&webgl.splat_shader));
    gl.bind_vertex_array(Some(&webgl.splat_vao));


    let proj_uniform_location = gl.get_uniform_location(&webgl.splat_shader, "projection").unwrap();
    gl.uniform_matrix4fv_with_f32_array(Some(&proj_uniform_location), false, vpm.as_slice());

    let camera_uniform_location = gl.get_uniform_location(&webgl.splat_shader, "camera").unwrap();
    gl.uniform_matrix4fv_with_f32_array(Some(&camera_uniform_location), false, vm.as_slice());


    let width = width as f32;
    let height = height as f32;
    let tan_fovy = f32::tan(0.820176 * 0.5);
    let tan_fovx = (tan_fovy * width) / height;
    let focal_y = height / (2.0 * tan_fovy);
    let focal_x = width / (2.0 * tan_fovx);
    set_float_uniform_value(&webgl.splat_shader, &gl, "W", width as f32);
    set_float_uniform_value(&webgl.splat_shader, &gl, "H", height as f32);
    set_float_uniform_value(&webgl.splat_shader, &gl, "focal_x", focal_x);
    set_float_uniform_value(&webgl.splat_shader, &gl, "focal_y", focal_y);
    set_float_uniform_value(&webgl.splat_shader, &gl, "tan_fovx", tan_fovx);
    set_float_uniform_value(&webgl.splat_shader, &gl, "tan_fovy", tan_fovy);

    // gl.active_texture(WebGl2RenderingContext::TEXTURE0);
    // gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&webgl.color_texture));
    
    // gl.active_texture(WebGl2RenderingContext::TEXTURE1);
    // gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&webgl.position_texture));
    // set_float_uniform_value(shader_program, gl, "scale_modifier", 1.0);

    // TODO: edit these
    // set_vec3_uniform_value(shader_program, gl, "boxmin", [-1.0, -1.0, -1.0]);
    // set_vec3_uniform_value(shader_program, gl, "boxmax", [1.0, 1.0, 1.0]);



 
    // let s = promptJS("eh;");
    // log!("s is {}", s);

    // Clear the canvas
    // gl.clear_color(0.5, 0.5, 0.5, 0.9);
    // gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear_color(0.0, 0.0, 0.0, 0.0);

    // Set the view port
    gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
    
    // Enable the depth test
    // gl.enable(WebGl2RenderingContext::DEPTH_TEST);

    // Clear the color buffer bit
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    // gl.clear_color(0.0, 0.0, 0.0, 1.0);

    gl.disable(WebGl2RenderingContext::DEPTH_TEST);
	gl.enable(WebGl2RenderingContext::BLEND);
	// gl.blend_func(WebGl2RenderingContext::ONE_MINUS_CONSTANT_ALPHA, WebGl2RenderingContext::ONE);
	gl.blend_func(WebGl2RenderingContext::ONE_MINUS_DST_ALPHA, WebGl2RenderingContext::ONE);

    // gl.enable(WebGl2RenderingContext::ALIASED_POINT_SIZE_RANGE);


    // Draw the triangle


    // gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
    let gaussian_count = num_vertices;

    // gl.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 6);
    // gl.draw_arrays(WebGl2RenderingContext::POINTS, 0, 3);
    gl.draw_arrays_instanced( WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4, gaussian_count);



    gl.use_program(Some(&webgl.geo_shader));
    gl.bind_vertex_array(Some(&webgl.geo_vao));
    gl.enable(WebGl2RenderingContext::DEPTH_TEST);
	gl.enable(WebGl2RenderingContext::BLEND);
    let proj_uniform_location = gl.get_uniform_location(&webgl.geo_shader, "projection").unwrap();
    gl.uniform_matrix4fv_with_f32_array(Some(&proj_uniform_location), false, vpm.as_slice());
    let camera_uniform_location = gl.get_uniform_location(&webgl.geo_shader, "camera").unwrap();
    gl.uniform_matrix4fv_with_f32_array(Some(&camera_uniform_location), false, vm.as_slice());

    gl.draw_arrays(TRIANGLES, 0, webgl.geo_count);
    // gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, settings.maxGaussians);

    return 
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn get_canvas_context(canvas_id: &str) -> web_sys::CanvasRenderingContext2d {
    let window = window();
    let document = window.document().expect("should have a document on window");
    let canvas = document
        .get_element_by_id(canvas_id)
        .expect("should have a canvas on the page")
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap()
}

pub fn get_world_to_camera_matrix(cam : &CameraInfo) -> Mat4{
    let mut camera: Mat4 = glm::identity();
    camera = glm::rotate_z(&camera, 1.0 * glm::pi::<f32>());
    camera = glm::rotate_x(&camera, cam.rot.x);
    camera = glm::rotate_y(&camera, cam.rot.y);
    camera = glm::translate(&camera, &cam.pos);
    return camera;
}

pub fn get_camera_to_world_matrix(cam : &CameraInfo) -> Mat4 {
    return get_world_to_camera_matrix(cam).try_inverse().unwrap();
}

pub fn get_scene_ready_for_draw(width: i32, height: i32, scene: &mut Scene, cam: &CameraInfo) -> (Mat4, Mat4, Vec<u32>){
    let _timer = Timer::new("get_scene_ready_for_draw");
    let mut proj = glm::perspective((width as f32)/ (height as f32), 0.820176f32, 0.1f32, 100f32);
    // camera = camera.try_inverse().unwrap();

    let mut vm = get_world_to_camera_matrix(&cam);
    let mut vpm = proj * vm;
    invertRow(&mut vm, 1);
    invertRow(&mut vm, 2);
    invertRow(&mut vpm, 1);
    invertRow(&mut vm, 0);
    invertRow(&mut vpm, 0);

    let splat_indices = scene.sort_splats_based_on_depth(vpm);
    return (vm, vpm, splat_indices)
}

pub struct CameraInfo{
    pub pos: Vec3,
    pub rot: Vec2,
    pub is_dragging: bool,
    pub last_mouse_pos: Vec2,
}

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[allow(non_snake_case)]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    set_panic_hook();
    // log!("web!");
    // let ply_splat = loader::loader::load_ply().await.expect("something went wrong in loading");
    // let ply_splat = loader::loader::load_ply().await.expect("something went wrong in loading");
    // let mut scene = Scene::new(ply_splat);
    // let serealized = serde_json::to_string(&scene).unwrap();
    // log!("serialized = {}", serealized);
    // log!("Done loading!");
    // Load the JSON file dynamically
    // let window = web_sys::window().unwrap();
    // let mut scene: Scene = Scene::new_from_url("http://127.0.0.1:5501/splats/one-corn.json").await;
    // let scene_name = "shahan_head";
    // let scene_name = "Shahan_03_id01-30000.cleaned";
    log!("Starting Web!");
    // unsafe {
    //     if !worker_initialized {
    //         // let worker_options = WorkerOptions::new();
    //         // worker_options.set_type(WorkerType::Module);

    //         //     let worker_handle = Rc::new(RefCell::new(Worker::new_with_options("./worker.js", &worker_options).unwrap()));
    //         // console::log_1(&"Created a new worker from within Wasm".into());
    //         // worker_handle.borrow_mut().post_message(&JsValue::from_str("hello from wasm")).unwrap();
    //         worker_initialized = true;
    //     }
    //     else {
    //         return Ok(());
    //     }
    // }

    let scene_name = "Shahan_03_id01-30000";
    // let scene_name = "Shahan_03_id01-30000.cleaned";
    let mut scene: Scene = Scene::new_from_url(&format!("http://127.0.0.1:5501/splats/{}.rkyv", scene_name)).await;
    // let mut scene: Scene = Scene::new_from_json(&loaded_file);
    // log!("deserialized = {:?}", scene);

    // let ply_splat = loader::loader::load_ply().await.expect("something went wrong in loading");
    // log!("Done loading!");




    let _timer = Timer::new("start function");

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let width = canvas.width() as i32;
    let height = canvas.height() as i32;
    let gl = getWebGLContext();


    let webgl = setup_webgl(gl, &scene).unwrap();
    let cam_info = Rc::new(RefCell::new(CameraInfo {
        pos: Vec3::new(0.0, 0.0, 5.0),
        rot: Vec2::new(0.0, 0.0),
        is_dragging: false,
        last_mouse_pos: Vec2::new(0.0, 0.0),
    }));

    let cam_info_clone = cam_info.clone();
    let move_speed = 0.05;
    let keys_pressed = Rc::new(RefCell::new(std::collections::HashSet::new()));
    
    let keys_pressed_clone = keys_pressed.clone();
    let keydown_cb = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
        keys_pressed_clone.borrow_mut().insert(e.key());
    }) as Box<dyn FnMut(_)>);
    document.add_event_listener_with_callback("keydown", keydown_cb.as_ref().unchecked_ref())?;
    keydown_cb.forget();

    let keys_pressed_clone = keys_pressed.clone();
    let keyup_cb = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
        keys_pressed_clone.borrow_mut().remove(&e.key());
    }) as Box<dyn FnMut(_)>);
    document.add_event_listener_with_callback("keyup", keyup_cb.as_ref().unchecked_ref())?;
    keyup_cb.forget();

    // Add mouse event handlers
    let cam_info_mousedown = cam_info.clone();
    let mousedown_cb = Closure::wrap(Box::new(move |e: MouseEvent| {
        let mut cam_info = cam_info_mousedown.borrow_mut();
        cam_info.is_dragging = true;
        cam_info.last_mouse_pos = Vec2::new(e.client_x() as f32, e.client_y() as f32);
    }) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("mousedown", mousedown_cb.as_ref().unchecked_ref())?;
    mousedown_cb.forget();

    let cam_info_mousemove = cam_info.clone();
    let mousemove_cb = Closure::wrap(Box::new(move |e: MouseEvent| {
        let mut cam_info = cam_info_mousemove.borrow_mut();
        if cam_info.is_dragging {
            let current_pos = Vec2::new(e.client_x() as f32, e.client_y() as f32);
            let delta = current_pos - cam_info.last_mouse_pos;
            
            // Adjust these factors to control rotation speed
            let rotation_factor_x = 0.001;
            let rotation_factor_y = 0.001;

            cam_info.rot.y -= -delta.x * rotation_factor_x;
            cam_info.rot.x -= -delta.y * rotation_factor_y;

            // Clamp vertical rotation to avoid flipping
            cam_info.rot.x = cam_info.rot.x.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

            cam_info.last_mouse_pos = current_pos;
        }
    }) as Box<dyn FnMut(_)>);
    document.add_event_listener_with_callback("mousemove", mousemove_cb.as_ref().unchecked_ref())?;
    mousemove_cb.forget();

    let cam_info_mouseup = cam_info.clone();
    let mouseup_cb = Closure::wrap(Box::new(move |_: MouseEvent| {
        let mut cam_info = cam_info_mouseup.borrow_mut();
        cam_info.is_dragging = false;
    }) as Box<dyn FnMut(_)>);
    document.add_event_listener_with_callback("mouseup", mouseup_cb.as_ref().unchecked_ref())?;
    mouseup_cb.forget();

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut i = 0;

    let cam_info_clone2 = cam_info.clone();
    let keys_pressed_clone = keys_pressed.clone();
    *g.borrow_mut() = Some(Closure::new(move || {
        let _timer = Timer::new("main loop");
        let mut cam_info = cam_info_clone2.borrow_mut();
        let keys = keys_pressed_clone.borrow();

        let mut cam_translation_local = vec3(0.0, 0.0, 0.0);

        if keys.contains("w") {
            cam_translation_local.z -= move_speed;
        }
        if keys.contains("s") {
            cam_translation_local.z += move_speed;
        }
        if keys.contains("a") {
            cam_translation_local.x += move_speed;
        }
        if keys.contains("d") {
            cam_translation_local.x -= move_speed;
        }
        if keys.contains(" ") {
            cam_translation_local.y -= move_speed;
        }
        if keys.contains("Shift") {
            cam_translation_local.y += move_speed;
        }

        if cam_translation_local != vec3(0.0, 0.0, 0.0) {
            let cam_to_world = get_camera_to_world_matrix(&cam_info);
            let cam_pos_after_moving = cam_to_world * vec4(cam_translation_local.x, cam_translation_local.y, cam_translation_local.z, 0.0);
            cam_info.pos += vec3(cam_pos_after_moving.x, cam_pos_after_moving.y, cam_pos_after_moving.z);
        }

        if keys.contains("ArrowUp") {
            cam_info.rot.x -= 0.1;
        }
        if keys.contains("ArrowDown") {
            cam_info.rot.x += 0.1;
        }
        if keys.contains("ArrowLeft") {
            cam_info.rot.y -= 0.1;
        }
        if keys.contains("ArrowRight") {
            cam_info.rot.y += 0.1;
        }

        let (vm, vpm, splat_indices) = get_scene_ready_for_draw(width, height, &mut scene, &cam_info);

        update_splat_indices(&scene, &webgl, &splat_indices);
        update_webgl_buffers(&scene, &webgl);
        draw(&webgl, &canvas, i, scene.splats.len() as i32, vpm, vm);

        i += 1;
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}


pub struct AppState{
    scene: Scene,
    webgl: WebGLSetupResult,
    canvas: HtmlCanvasElement,
    width: i32,
    height: i32,
    i: i32,
}

// #[wasm_bindgen]
// impl AppState {
//     #[wasm_bindgen(constructor)]
//     pub async fn new() -> Result<Rc<RefCell<AppState>>, JsValue> {
//         let ply_splat = loader::loader::load_ply().await.expect("something went wrong in loading");
//         set_panic_hook(); 
//         let mut scene = Scene::new(ply_splat);

//         let document = web_sys::window().unwrap().document().unwrap();
//         let canvas = document.get_element_by_id("canvas").unwrap();
//         let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

//         let width = canvas.width() as i32;
//         let height = canvas.height() as i32;
//         let gl = getWebGLContext();


//         let webgl = setup_webgl(gl, &scene).unwrap();

//         Ok(Rc::new(RefCell::new(AppState {
//             scene,
//             webgl,
//             canvas,
//             width,
//             height,
//             i: 0,
//         })))
//     }

//     // pub fn handle_keyboard(&mut self, key: &str) {
//     //     log!("Key pressed: {}", key);

//     //     // Update the scene's view matrix or any other state based on the key input
//     //     match key {
//     //         "ArrowUp" => {
//     //             // Example: Adjust the view matrix for upward arrow key
//     //             self.scene.view_matrix[1][3] += 0.1;
//     //         }
//     //         "ArrowDown" => {
//     //             // Example: Adjust the view matrix for downward arrow key
//     //             self.scene.view_matrix[1][3] -= 0.1;
//     //         }
//     //         // Add more key handling as needed
//     //         _ => {}
//     //     }
//     // }

//     // pub fn render_frame(&mut self) {
//     //     let (vm, vpm) = get_scene_ready_for_draw(self.width, self.height, &mut self.scene);
//     //     update_webgl_buffers(&self.scene, &self.webgl);
//     //     draw(&self.webgl.gl, &self.webgl.program, &self.canvas, self.i, self.scene.splats.len() as i32, vpm, vm);
//     //     self.i += 1;
//     // }
// }

// #[wasm_bindgen(start)]
// pub async fn start() -> Result<Rc<RefCell<AppState>>, JsValue> {
//     set_panic_hook();
//     AppState::new().await
// }
