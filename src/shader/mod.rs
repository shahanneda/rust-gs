pub mod shader {
    use web_sys::{WebGl2RenderingContext, WebGlProgram};

    use crate::utils::{compile_shader, link_program};

    pub fn create_shader_program(
        gl: &WebGl2RenderingContext,
        vert_code: &str,
        frag_code: &str,
    ) -> Result<WebGlProgram, String> {
        let vert_shader = compile_shader(&gl, WebGl2RenderingContext::VERTEX_SHADER, vert_code)?;
        let frag_shader = compile_shader(&gl, WebGl2RenderingContext::FRAGMENT_SHADER, frag_code)?;
        let shader_program = link_program(&gl, &vert_shader, &frag_shader)?;

        return Ok(shader_program);
    }
}
