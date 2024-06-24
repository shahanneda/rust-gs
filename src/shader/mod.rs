pub mod shader {
    use web_sys::{WebGl2RenderingContext, WebGlProgram};

    use crate::utils::{compile_shader, link_program};

	pub fn create_splat_shader_program(gl: &WebGl2RenderingContext) -> Result<WebGlProgram, String>{
        let vert_code = include_str!("./basic_vert.glsl");
        let vert_shader = compile_shader(&gl, WebGl2RenderingContext::VERTEX_SHADER, vert_code)?;
        let frag_code = include_str!("./basic_frag.glsl");
        let frag_shader = compile_shader(&gl, WebGl2RenderingContext::FRAGMENT_SHADER, frag_code)?;
        let shader_program = link_program(&gl, &vert_shader, &frag_shader)?;

        return Ok(shader_program);
	}

	pub fn create_geo_shader_program(gl: &WebGl2RenderingContext) -> Result<WebGlProgram, String>{
        let vert_code = include_str!("./geo_vert.glsl");
        let vert_shader = compile_shader(&gl, WebGl2RenderingContext::VERTEX_SHADER, vert_code)?;
        let frag_code = include_str!("./geo_frag.glsl");
        let frag_shader = compile_shader(&gl, WebGl2RenderingContext::FRAGMENT_SHADER, frag_code)?;
        let shader_program = link_program(&gl, &vert_shader, &frag_shader)?;

        return Ok(shader_program);
	}
}