use js_sys::Uint32Array;

use crate::log;
use crate::scene::Scene;
use crate::scene_object::SceneObject;
use crate::shader;
use crate::timer::Timer;
extern crate eframe;
extern crate js_sys;
extern crate nalgebra_glm as glm;
extern crate ply_rs;
extern crate wasm_bindgen;
use crate::scene_geo;
use crate::utils::float32_array_from_vec;
use crate::utils::uint32_array_from_vec;
use crate::web::Settings;
use js_sys::Float32Array;
use std::convert::TryInto;
use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext;
use web_sys::WebGlBuffer;
use web_sys::WebGlProgram;
use web_sys::WebGlTexture;
use web_sys::WebGlUniformLocation;
use web_sys::WebGlVertexArrayObject;

struct SplatTexture {
    color_texture: WebGlTexture,
    position_texture: WebGlTexture,
    cov3da_texture: WebGlTexture,
    cov3db_texture: WebGlTexture,
    opacity_texture: WebGlTexture,
    offset: u32,
}

pub struct Renderer {
    gl: WebGl2RenderingContext,
    splat_shader: WebGlProgram,
    splat_vao: WebGlVertexArrayObject,
    geo_shader: WebGlProgram,
    geo_vertex_buffer: WebGlBuffer,
    geo_color_buffer: WebGlBuffer,
    geo_index_buffer: WebGlBuffer,
    geo_normal_buffer: WebGlBuffer,
    splat_index_buffer: WebGlBuffer,
    geo_count: i32,
    geo_vao: WebGlVertexArrayObject,
    line_vao: WebGlVertexArrayObject,
    line_vertex_buffer: WebGlBuffer,
    line_color_buffer: WebGlBuffer,
    line_shader: WebGlProgram,
    splat_textures: Vec<SplatTexture>,
}

impl Renderer {
    pub fn new(gl: WebGl2RenderingContext, scene: &Scene) -> Result<Renderer, JsValue> {
        // let vertices: [f32; 3 * 4] = [
        //     -1.0, -1.0, 0.0, //
        //     1.0, -1.0, 0.0, //
        //     -1.0, 1.0, 0.0, //
        //     1.0, 1.0, 0.0, //
        // ];
        // let vertices = vertices.map(|v| v);

        // Create Splat VAO and buffers
        let splat_vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&splat_vao));
        let splat_index_buffer = create_buffer(&gl).unwrap();
        // END SPLAT VAO AND BUFFERS

        // Create Geo VAO and buffers
        let geo_vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&geo_vao));

        let geo_vertex_buffer = create_buffer(&gl).unwrap();
        let vertices = scene_geo::CUBE_VERTICES.map(|v| v);

        update_buffer_data(&gl, &geo_vertex_buffer, float32_array_from_vec(&vertices));
        let geo_color_buffer = create_buffer(&gl).unwrap();
        update_buffer_data(
            &gl,
            &geo_color_buffer,
            float32_array_from_vec(&scene_geo::CUBE_COLORS),
        );

        let geo_index_buffer = create_buffer(&gl).unwrap();
        update_buffer_data_u32(
            &gl,
            &geo_index_buffer,
            uint32_array_from_vec(&scene_geo::CUBE_INDICES),
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
        );

        let geo_normal_buffer = create_buffer(&gl).unwrap();
        update_buffer_data(
            &gl,
            &geo_normal_buffer,
            float32_array_from_vec(&scene_geo::CUBE_NORMALS),
        );

        // END GEO VAO

        // Create Shaders
        let splat_shader = shader::shader::create_shader_program(
            &gl,
            include_str!("./shader/splat_vert.glsl"),
            include_str!("./shader/splat_frag.glsl"),
        )
        .unwrap();
        let geo_shader = shader::shader::create_shader_program(
            &gl,
            include_str!("./shader/geo_vert.glsl"),
            include_str!("./shader/geo_frag.glsl"),
        )
        .unwrap();
        let line_shader = shader::shader::create_shader_program(
            &gl,
            include_str!("./shader/line_vert.glsl"),
            include_str!("./shader/line_frag.glsl"),
        )
        .unwrap();
        // END SHADERS

        // Activate splat vao
        gl.bind_vertex_array(Some(&splat_vao));
        let mut splat_textures = Vec::new();
        for scene_no in [0, 1] {
            let offset = scene_no * 5 as u32;
            gl.active_texture(WebGl2RenderingContext::TEXTURE0 + offset);
            let (color_texture, color_texture_location) = create_texture(
                &gl,
                &splat_shader,
                "u_color_texture",
                WebGl2RenderingContext::TEXTURE0 + offset + COLOR_TEXTURE_UNIT,
            )?;

            gl.active_texture(WebGl2RenderingContext::TEXTURE0 + offset + POSITION_TEXTURE_UNIT);
            let (position_texture, position_texture_location) = create_texture(
                &gl,
                &splat_shader,
                "u_position_texture",
                WebGl2RenderingContext::TEXTURE0 + POSITION_TEXTURE_UNIT,
            )?;
            let (cov3da_texture, cov3da_texture_location) = create_texture(
                &gl,
                &splat_shader,
                "u_cov3da_texture",
                WebGl2RenderingContext::TEXTURE0 + offset + COV3DA_TEXTURE_UNIT,
            )?;
            let (cov3db_texture, cov3db_texture_location) = create_texture(
                &gl,
                &splat_shader,
                "u_cov3db_texture",
                WebGl2RenderingContext::TEXTURE0 + offset + COV3DB_TEXTURE_UNIT,
            )?;
            let (opacity_texture, opacity_texture_location) = create_texture(
                &gl,
                &splat_shader,
                "u_opacity_texture",
                WebGl2RenderingContext::TEXTURE0 + offset + OPACITY_TEXTURE_UNIT,
            )?;

            let splat_texture = SplatTexture {
                color_texture,
                position_texture,
                cov3da_texture,
                cov3db_texture,
                opacity_texture,
                offset: 0,
            };
            splat_textures.push(splat_texture);
        }

        let line_vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&line_vao));
        let line_vertex_buffer = create_buffer(&gl).unwrap();
        update_buffer_data(
            &gl,
            &line_vertex_buffer,
            float32_array_from_vec(&vec![0.0, 0.0, 0.0]),
        );
        let line_color_buffer = create_buffer(&gl).unwrap();
        update_buffer_data(
            &gl,
            &line_color_buffer,
            float32_array_from_vec(&vec![1.0, 0.0, 0.0]),
        );

        let result = Renderer {
            gl: gl,
            splat_shader,
            splat_vao,
            geo_shader,
            geo_vertex_buffer,
            geo_color_buffer,
            geo_index_buffer,
            geo_normal_buffer,
            splat_index_buffer,
            geo_count: vertices.len() as i32 / 3,
            geo_vao,
            line_vao,
            line_vertex_buffer,
            line_color_buffer,
            line_shader,
            splat_textures,
        };

        result.gl.use_program(Some(&result.splat_shader));
        result.gl.bind_vertex_array(Some(&result.splat_vao));
        // result.bind_splat_textures(0);

        result
            .update_webgl_textures(&scene, 0)
            .expect("failed to update webgl textures for the first time!");

        create_attribute_and_get_location(
            &result.gl,
            &result.splat_index_buffer,
            &result.splat_shader,
            "s_index",
            true,
            1,
            WebGl2RenderingContext::UNSIGNED_INT,
        );

        result.gl.use_program(Some(&result.geo_shader));
        result.gl.bind_vertex_array(Some(&result.geo_vao));
        create_attribute_and_get_location(
            &result.gl,
            &result.geo_vertex_buffer,
            &result.geo_shader,
            "v_pos",
            false,
            3,
            WebGl2RenderingContext::FLOAT,
        );
        create_attribute_and_get_location(
            &result.gl,
            &result.geo_color_buffer,
            &result.geo_shader,
            "v_col",
            false,
            3,
            WebGl2RenderingContext::FLOAT,
        );
        create_attribute_and_get_location(
            &result.gl,
            &result.geo_normal_buffer,
            &result.geo_shader,
            "v_norm",
            false,
            3,
            WebGl2RenderingContext::FLOAT,
        );

        result.gl.use_program(Some(&result.line_shader));
        result.gl.bind_vertex_array(Some(&result.line_vao));
        create_attribute_and_get_location(
            &result.gl,
            &result.line_vertex_buffer,
            &result.line_shader,
            "v_pos",
            false,
            3,
            WebGl2RenderingContext::FLOAT,
        );
        create_attribute_and_get_location(
            &result.gl,
            &result.line_color_buffer,
            &result.line_shader,
            "v_col",
            false,
            3,
            WebGl2RenderingContext::FLOAT,
        );

        return Ok(result);
    }

    pub fn bind_splat_textures(self: &Renderer, scene_idx: usize) {
        let offset = scene_idx as u32 * 5;
        self.gl.use_program(Some(&self.splat_shader));

        self.gl
            .pixel_storei(WebGl2RenderingContext::UNPACK_ALIGNMENT, 1);

        let texture_bindings = [
            (
                COLOR_TEXTURE_UNIT,
                &self.splat_textures[scene_idx].color_texture,
                "u_color_texture",
            ),
            (
                POSITION_TEXTURE_UNIT,
                &self.splat_textures[scene_idx].position_texture,
                "u_position_texture",
            ),
            (
                COV3DA_TEXTURE_UNIT,
                &self.splat_textures[scene_idx].cov3da_texture,
                "u_cov3da_texture",
            ),
            (
                COV3DB_TEXTURE_UNIT,
                &self.splat_textures[scene_idx].cov3db_texture,
                "u_cov3db_texture",
            ),
            (
                OPACITY_TEXTURE_UNIT,
                &self.splat_textures[scene_idx].opacity_texture,
                "u_opacity_texture",
            ),
        ];

        for (unit, texture, uniform_name) in texture_bindings {
            self.gl
                .active_texture(WebGl2RenderingContext::TEXTURE0 + offset + unit);
            self.gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
            set_texture_uniform_value(
                &self.splat_shader,
                &self.gl,
                uniform_name,
                texture,
                offset + unit,
            );
        }
    }

    pub fn draw_scene(
        self: &Renderer,
        canvas: &web_sys::HtmlCanvasElement,
        scene: &Scene,
        vpm: glm::Mat4,
        vm: glm::Mat4,
        normal_projection_matrix: glm::Mat4,
        normal_view_matrix: glm::Mat4,
        settings: &Settings,
        do_clear: bool,
    ) {
        let gl = &self.gl;
        let width = canvas.width() as i32;
        let height = canvas.height() as i32;
        let mut scale = 1.0;
        if settings.view_individual_splats {
            scale = 0.1;
        }

        // // turn on depth testing for normal objects
        gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        gl.depth_func(WebGl2RenderingContext::LEQUAL);

        if do_clear {
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            // Clear the color buffer bit
            gl.clear(
                WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
            );
        }

        self.draw_splat(
            width,
            height,
            scene.splat_data.splats.len() as i32,
            vpm,
            vm,
            scale,
            settings.do_blending,
            do_clear,
            scene.model_transform,
        );

        // draw scene objects
        for object in &scene.objects {
            self.draw_geo(
                width,
                height,
                vpm,
                vm,
                object,
                false,
                scene.light_pos,
                false,
                [0.0, 0.0, 0.0].into(),
                true,
            );
        }

        if scene.gizmo.target_object.is_some() {
            gl.disable(WebGl2RenderingContext::DEPTH_TEST);
            // gl.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);
            for axis_object in scene.gizmo.get_axis_objects() {
                self.draw_geo(
                    width,
                    height,
                    vpm,
                    vm,
                    axis_object,
                    false,
                    scene.light_pos,
                    false,
                    [0.0, 0.0, 0.0].into(),
                    false,
                );
            }
            gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        }

        if settings.show_octtree {
            self.draw_geo(
                width,
                height,
                vpm,
                vm,
                &scene.line_mesh,
                true,
                scene.light_pos,
                false,
                [0.0, 0.0, 0.0].into(),
                false,
            );
        }
    }

    // returns (index, is_gizmo, hit_object)
    pub fn get_at_mouse_position(
        self: &Renderer,
        width: i32,
        height: i32,
        mouse_x: i32,
        mouse_y: i32,
        vpm: glm::Mat4,
        vm: glm::Mat4,
        scene: &Scene,
    ) -> (u32, bool, bool) {
        let gl = &self.gl;
        self.draw_picking(width, height, vpm, vm, scene);
        // let mouse_x = mouse_x / width as f32;
        // let mouse_y = mouse_y / height as f32;
        // glm::vec3(mouse_x, mouse_y, 0.0)
        let mut buffer: [u8; 4] = [0, 0, 0, 0];
        gl.read_buffer(WebGl2RenderingContext::BACK);

        let y = height - mouse_y;
        gl.read_pixels_with_opt_u8_array(
            mouse_x as i32,
            y as i32,
            1,
            1,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(&mut buffer),
        )
        .unwrap();
        log!("buffer: {:?}", buffer);
        if buffer[1] != 0 {
            return (0, false, false);
        }

        if buffer[2] == 0 {
            let index = buffer[0] as u32;
            return (index, false, true);
        }

        // is gizmo
        return (buffer[0] as u32, true, true);
    }

    pub fn draw_picking(
        self: &Renderer,
        width: i32,
        height: i32,
        vpm: glm::Mat4,
        vm: glm::Mat4,
        scene: &Scene,
    ) {
        let gl = &self.gl;
        gl.clear_color(0.0, 1.0, 0.0, 0.0);
        gl.clear(
            WebGl2RenderingContext::DEPTH_BUFFER_BIT | WebGl2RenderingContext::COLOR_BUFFER_BIT,
        );

        for (index, object) in scene.objects.iter().enumerate() {
            self.draw_geo(
                width,
                height,
                vpm,
                vm,
                object,
                false,
                scene.light_pos,
                true,
                [index as f32 / 256.0, 0.0, 0.0].into(),
                false,
            );
        }

        if scene.gizmo.target_object.is_some() {
            gl.disable(WebGl2RenderingContext::DEPTH_TEST);
            // gl.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);
            for (index, axis_object) in scene.gizmo.get_axis_objects().iter().enumerate() {
                self.draw_geo(
                    width,
                    height,
                    vpm,
                    vm,
                    axis_object,
                    false,
                    scene.light_pos,
                    true,
                    [index as f32 / 256.0, 0.0, 1.0].into(),
                    false,
                );
            }
            gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        }
    }

    pub fn draw_splat(
        self: &Renderer,
        width: i32,
        height: i32,
        num_vertices: i32,
        vpm: glm::Mat4,
        vm: glm::Mat4,
        scale: f32,
        do_blending: bool,
        do_clear: bool,
        model_transform: glm::Mat4,
    ) {
        let gl = &self.gl;
        gl.use_program(Some(&self.splat_shader));
        gl.bind_vertex_array(Some(&self.splat_vao));

        set_matrix4_uniform_value(&self.splat_shader, &gl, "projection", vpm);
        set_matrix4_uniform_value(&self.splat_shader, &gl, "camera", vm);
        set_matrix4_uniform_value(&self.splat_shader, &gl, "model", model_transform);

        let width = width as f32;
        let height = height as f32;
        let tan_fovy = f32::tan(0.820176 * 0.5);
        let tan_fovx = (tan_fovy * width) / height;
        let focal_y = height / (2.0 * tan_fovy);
        let focal_x = width / (2.0 * tan_fovx);
        set_float_uniform_value(&self.splat_shader, &gl, "W", width as f32);
        set_float_uniform_value(&self.splat_shader, &gl, "H", height as f32);
        set_float_uniform_value(&self.splat_shader, &gl, "focal_x", focal_x);
        set_float_uniform_value(&self.splat_shader, &gl, "focal_y", focal_y);
        set_float_uniform_value(&self.splat_shader, &gl, "tan_fovx", tan_fovx);
        set_float_uniform_value(&self.splat_shader, &gl, "tan_fovy", tan_fovy);
        set_float_uniform_value(&self.splat_shader, &gl, "scale", scale);
        set_bool_uniform_value(&self.splat_shader, &gl, "do_blending", do_blending);

        gl.viewport(0, 0, width as i32, height as i32);

        gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        gl.depth_func(WebGl2RenderingContext::ALWAYS);
        gl.depth_mask(true);

        gl.enable(WebGl2RenderingContext::BLEND);
        if do_clear {
            gl.blend_func(
                WebGl2RenderingContext::ONE_MINUS_DST_ALPHA,
                WebGl2RenderingContext::ONE,
            );
        } else {
            gl.blend_func(
                WebGl2RenderingContext::SRC_ALPHA,
                WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
            );
        }
        // gl.clear_depth(1.0);
        // gl.blend_func(
        //     WebGl2RenderingContext::ONE_MINUS_DST_ALPHA,
        //     WebGl2RenderingContext::ONE,
        // );
        let gaussian_count = num_vertices;
        gl.draw_arrays_instanced(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4, gaussian_count);
    }

    pub fn draw_geo(
        self: &Renderer,
        width: i32,
        height: i32,
        vpm: glm::Mat4,
        vm: glm::Mat4,
        object: &SceneObject,
        is_line: bool,
        light_pos: glm::Vec3,
        is_picking: bool,
        picking_color: glm::Vec3,
        shadows: bool,
    ) {
        let gl = &self.gl;
        gl.use_program(Some(&self.geo_shader));
        gl.bind_vertex_array(Some(&self.geo_vao));
        if !is_line {
            gl.enable(WebGl2RenderingContext::DEPTH_TEST);
            gl.depth_func(WebGl2RenderingContext::LEQUAL);
            // gl.disable(WebGl2RenderingContext::DEPTH_TEST);
            gl.depth_mask(true);
            gl.enable(WebGl2RenderingContext::CULL_FACE);
        } else {
            gl.disable(WebGl2RenderingContext::DEPTH_TEST);
            gl.depth_mask(false);
        }
        // gl.depth_func(WebGl2RenderingContext::GEQUAL);

        gl.disable(WebGl2RenderingContext::BLEND);

        // gl.enable(WebGl2RenderingContext::BLEND);
        update_buffer_data(
            &gl,
            &self.geo_vertex_buffer,
            float32_array_from_vec(&object.mesh_data.vertices),
        );
        update_buffer_data(
            &gl,
            &self.geo_color_buffer,
            float32_array_from_vec(&object.mesh_data.colors),
        );
        update_buffer_data_u32(
            &gl,
            &self.geo_index_buffer,
            uint32_array_from_vec(&object.mesh_data.indices),
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
        );
        update_buffer_data(
            &gl,
            &self.geo_normal_buffer,
            float32_array_from_vec(&object.mesh_data.normals),
        );

        // log!("indicies are {:?}", object.mesh_data.indices);
        // log!("colors length: {:?}", scene_geo::COLORS.len());
        // gl.blend_func(
        //     WebGl2RenderingContext::ONE_MINUS_DST_ALPHA,
        //     WebGl2RenderingContext::ONE,
        // );

        let proj_uniform_location = gl
            .get_uniform_location(&self.geo_shader, "projection")
            .unwrap();
        gl.uniform_matrix4fv_with_f32_array(Some(&proj_uniform_location), false, vpm.as_slice());

        // try muliplying just for checking
        // for vertex in scene_geo::PYRAMID_VERTICES.chunks(3) {
        //     // after vpm
        //     log!("vertex: {:?}", vertex);
        //     let vertex_vpm = vpm * glm::vec4(vertex[0], vertex[1], vertex[2], 1.0);
        //     log!("vertex_vpm: {:?}", vertex_vpm);
        // }

        let camera_uniform_location = gl.get_uniform_location(&self.geo_shader, "camera").unwrap();
        gl.uniform_matrix4fv_with_f32_array(Some(&camera_uniform_location), false, vm.as_slice());

        let light_pos_uniform_location = gl
            .get_uniform_location(&self.geo_shader, "light_pos")
            .unwrap();
        gl.uniform3fv_with_f32_array(Some(&light_pos_uniform_location), light_pos.as_slice());

        set_bool_uniform_value(&self.geo_shader, &gl, "is_picking", is_picking);
        set_bool_uniform_value(&self.geo_shader, &gl, "shadows", shadows);
        set_vec3_uniform_value(
            &self.geo_shader,
            &gl,
            "picking_color",
            [picking_color.x, picking_color.y, picking_color.z],
        );

        let model = object.get_transform();

        let model_uniform_location = gl.get_uniform_location(&self.geo_shader, "model").unwrap();
        if !is_line {
            gl.uniform_matrix4fv_with_f32_array(
                Some(&model_uniform_location),
                false,
                model.as_slice(),
            );
        } else {
            gl.uniform_matrix4fv_with_f32_array(
                Some(&model_uniform_location),
                false,
                glm::identity::<f32, 4>().as_slice(),
            );
        }

        gl.bind_buffer(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&self.geo_index_buffer),
        );
        if !is_line {
            gl.draw_elements_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                object.mesh_data.indices.len() as i32,
                WebGl2RenderingContext::UNSIGNED_INT,
                0,
            );
        } else {
            gl.draw_arrays(
                WebGl2RenderingContext::LINES,
                0,
                object.mesh_data.vertices.len() as i32 / 3,
            );
        }
    }

    pub fn draw_lines(
        self: &Renderer,
        width: i32,
        height: i32,
        line_verts: &Vec<f32>,
        line_colors: &Vec<f32>,
        view_mat: glm::Mat4,
        projection_mat: glm::Mat4,
    ) {
        let gl = &self.gl;
        // log!("about to draw lines");
        gl.use_program(Some(&self.geo_shader));
        gl.bind_vertex_array(Some(&self.geo_vao));
        gl.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.geo_color_buffer),
        );
        gl.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.geo_vertex_buffer),
        );

        gl.disable(WebGl2RenderingContext::DEPTH_TEST);
        gl.depth_mask(false);
        gl.disable(WebGl2RenderingContext::BLEND);
        gl.disable(WebGl2RenderingContext::CULL_FACE);
        update_buffer_data(
            &gl,
            &self.geo_vertex_buffer,
            float32_array_from_vec(line_verts),
        );

        update_buffer_data(
            &gl,
            &self.geo_color_buffer,
            float32_array_from_vec(line_colors),
        );
        // multipling the line as a sample
        // for i in 0..line_verts.len() / 3 {
        //     let vertex = glm::vec3(
        //         line_verts[i * 3],
        //         line_verts[i * 3 + 1],
        //         line_verts[i * 3 + 2],
        //     );
        //     let vertex_proj =
        //         projection_mat * view_mat * glm::vec4(vertex[0], vertex[1], vertex[2], 1.0);
        //     log!("vertex_proj: {:?}", vertex_proj);
        // }

        let proj_uniform_location = gl
            .get_uniform_location(&self.geo_shader, "projection")
            .unwrap();
        gl.uniform_matrix4fv_with_f32_array(
            Some(&proj_uniform_location),
            false,
            projection_mat.as_slice(),
        );

        let camera_uniform_location = gl.get_uniform_location(&self.geo_shader, "camera").unwrap();
        gl.uniform_matrix4fv_with_f32_array(
            Some(&camera_uniform_location),
            false,
            view_mat.as_slice(),
        );

        let model_uniform_location = gl.get_uniform_location(&self.geo_shader, "model").unwrap();
        gl.uniform_matrix4fv_with_f32_array(
            Some(&model_uniform_location),
            false,
            glm::identity::<f32, 4>().as_slice(),
        );

        set_float_uniform_value(&self.geo_shader, &gl, "W", width as f32);
        set_float_uniform_value(&self.geo_shader, &gl, "H", height as f32);

        // try muliplying just for checking
        // for vertex in scene_geo::PYRAMID_VERTICES.chunks(3) {
        //     // after vpm
        //     log!("vertex: {:?}", vertex);
        //     let vertex_vpm = vpm * glm::vec4(vertex[0], vertex[1], vertex[2], 1.0);
        //     log!("vertex_vpm: {:?}", vertex_vpm);
        // }

        gl.draw_arrays(
            WebGl2RenderingContext::LINES,
            0,
            line_verts.len() as i32 / 3,
        );
    }

    pub fn update_splat_indices(self: &Renderer, splat_indices: &Vec<u32>) {
        let _timer = Timer::new("update_splat_indices");
        self.gl.use_program(Some(&self.splat_shader));
        self.gl.bind_vertex_array(Some(&self.splat_vao));
        update_buffer_data_u32(
            &self.gl,
            &self.splat_index_buffer,
            uint32_array_from_vec(&splat_indices),
            WebGl2RenderingContext::ARRAY_BUFFER,
        );
    }

    pub fn update_webgl_textures(
        self: &Renderer,
        scene: &Scene,
        scene_idx: usize,
    ) -> Result<(), JsValue> {
        let offset = scene_idx * 5;
        let mut texture_data = vec![
            (COLOR_TEXTURE_UNIT, Vec::new()),    // colors
            (POSITION_TEXTURE_UNIT, Vec::new()), // positions
            (COV3DA_TEXTURE_UNIT, Vec::new()),   // cov3da
            (COV3DB_TEXTURE_UNIT, Vec::new()),   // cov3db
            (OPACITY_TEXTURE_UNIT, Vec::new()),  // opacities
        ];

        for s in &scene.splat_data.splats {
            texture_data[0].1.extend_from_slice(&[s.r, s.g, s.b]);
            texture_data[1].1.extend_from_slice(&[s.x, s.y, s.z]);
            texture_data[2]
                .1
                .extend_from_slice(&[s.cov3d[0], s.cov3d[1], s.cov3d[2]]);
            texture_data[3]
                .1
                .extend_from_slice(&[s.cov3d[3], s.cov3d[4], s.cov3d[5]]);
            texture_data[4].1.extend_from_slice(&[s.opacity, 0.0, 0.0]);
        }

        let texture_bindings = [
            (
                &self.splat_textures[scene_idx].color_texture,
                &texture_data[0].1,
            ),
            (
                &self.splat_textures[scene_idx].position_texture,
                &texture_data[1].1,
            ),
            (
                &self.splat_textures[scene_idx].cov3da_texture,
                &texture_data[2].1,
            ),
            (
                &self.splat_textures[scene_idx].cov3db_texture,
                &texture_data[3].1,
            ),
            (
                &self.splat_textures[scene_idx].opacity_texture,
                &texture_data[4].1,
            ),
        ];

        for (i, (texture, data)) in texture_bindings.iter().enumerate() {
            self.gl
                .active_texture(WebGl2RenderingContext::TEXTURE0 + (offset + i) as u32);
            put_data_into_texture(&self.gl, texture, &float32_array_from_vec(data))?;
        }

        Ok(())
    }
}

fn update_buffer_data(gl: &WebGl2RenderingContext, buffer: &WebGlBuffer, data: Float32Array) {
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &data,
        WebGl2RenderingContext::STATIC_DRAW,
    );
}

fn update_buffer_data_u32(
    gl: &WebGl2RenderingContext,
    buffer: &WebGlBuffer,
    data: Uint32Array,
    type_: u32,
) {
    gl.bind_buffer(type_, Some(&buffer));
    gl.buffer_data_with_array_buffer_view(type_, &data, WebGl2RenderingContext::STATIC_DRAW);
}

fn create_buffer(gl: &WebGl2RenderingContext) -> Result<WebGlBuffer, &'static str> {
    let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
    return Ok(buffer);
}

fn create_attribute_and_get_location(
    gl: &WebGl2RenderingContext,
    buffer: &WebGlBuffer,
    program: &WebGlProgram,
    name: &str,
    divisor: bool,
    size: i32,
    type_: u32,
) -> u32 {
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
    } else {
        panic!("Invalid type for attribute");
    }
    if divisor {
        gl.vertex_attrib_divisor(coord, 1);
    }
    return coord;
}

fn create_texture(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    name: &str,
    active_texture: u32,
) -> Result<(WebGlTexture, WebGlUniformLocation), JsValue> {
    let texture = gl.create_texture().ok_or("Failed to create texture")?;
    gl.active_texture(active_texture);
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

    let empty_array = Float32Array::new_with_length(0);
    put_data_into_texture(&gl, &texture, &empty_array)?;
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MIN_FILTER,
        WebGl2RenderingContext::NEAREST as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MAG_FILTER,
        WebGl2RenderingContext::NEAREST as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_WRAP_S,
        WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_WRAP_T,
        WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );

    let location = gl
        .get_uniform_location(program, name)
        .ok_or("Failed to get uniform location")?;

    Ok((texture, location))
}

const TEXTURE_WIDTH: i32 = 2000;

fn put_data_into_texture(
    gl: &WebGl2RenderingContext,
    texture: &WebGlTexture,
    data_array: &Float32Array,
) -> Result<(), JsValue> {
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));

    let level = 0;
    let internal_format = WebGl2RenderingContext::RGB32F as i32;
    let width = TEXTURE_WIDTH;
    let number_of_values = data_array.length() as i32;
    // We add Texture_width -1 so that we always do a ceiling division
    let height = (number_of_values + TEXTURE_WIDTH - 1) / TEXTURE_WIDTH; // Assuming 3 components (RGB) per pixel

    // resize data array to match the texture size
    // TODO: don't duplicat the array here, make sure arrays are the right size before passing into here
    let resized_data_array =
        Float32Array::new_with_length((TEXTURE_WIDTH * height * 3).try_into().unwrap());
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

fn set_float_uniform_value(
    shader_program: &WebGlProgram,
    gl: &WebGl2RenderingContext,
    name: &str,
    value: f32,
) {
    // log!("name: {}", name);
    let uniform_location = gl.get_uniform_location(&shader_program, name).unwrap();
    gl.uniform1f(Some(&uniform_location), value);
}

fn set_bool_uniform_value(
    shader_program: &WebGlProgram,
    gl: &WebGl2RenderingContext,
    name: &str,
    value: bool,
) {
    let uniform_location = gl.get_uniform_location(&shader_program, name).unwrap();
    gl.uniform1i(Some(&uniform_location), value as i32);
}

fn set_matrix4_uniform_value(
    shader_program: &WebGlProgram,
    gl: &WebGl2RenderingContext,
    name: &str,
    value: glm::Mat4,
) {
    let uniform_location = gl.get_uniform_location(&shader_program, name).unwrap();
    gl.uniform_matrix4fv_with_f32_array(Some(&uniform_location), false, value.as_slice());
}

fn set_vec3_uniform_value(
    shader_program: &WebGlProgram,
    gl: &WebGl2RenderingContext,
    name: &str,
    value: [f32; 3],
) {
    // log!("name: {}", name);
    let uniform_location = gl.get_uniform_location(&shader_program, name).unwrap();
    gl.uniform3fv_with_f32_array(Some(&uniform_location), value.as_slice());
}

pub fn set_texture_uniform_value(
    shader_program: &WebGlProgram,
    gl: &WebGl2RenderingContext,
    name: &str,
    texture: &WebGlTexture,
    active_texture: u32,
) {
    let uniform_location = gl.get_uniform_location(&shader_program, name).unwrap();
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
    gl.uniform1i(Some(&uniform_location), active_texture as i32);
}

// fn update_webgl_buffers(scene: &Scene, webgl: &WebGLSetupResult) {
// let _timer = Timer::new("update_webgl_buffers");
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
// }
