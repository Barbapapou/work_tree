use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlTexture};

#[derive(Clone)]
pub struct Material {
    pub shader: WebGlProgram,
    pub texture: WebGlTexture,
}

impl Material {
    pub fn bind(&self, gl: &WebGlRenderingContext) {
        gl.use_program(Some(&self.shader));
        gl.active_texture(WebGlRenderingContext::TEXTURE - 1);
        gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&self.texture));
        let u_sampler_location = gl
            .get_uniform_location(&self.shader, "uSampler")
            .expect("can't get uSampler location");
        gl.uniform1i(Some(&u_sampler_location), 0);
    }

    pub fn new(shader: WebGlProgram, texture: WebGlTexture) -> Material {
        Material {
            shader,
            texture,
        }
    }
}