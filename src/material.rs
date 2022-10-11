use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlTexture};
use crate::Renderer;

#[derive(Clone)]
pub struct Material {
    pub shader: WebGlProgram,
    pub texture: WebGlTexture,
}

impl Material {
    pub fn bind(&self, renderer: &Renderer) {
        let gl = &renderer.gl;
        gl.use_program(Some(&self.shader));
        gl.active_texture(WebGlRenderingContext::TEXTURE - 1);
        gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&self.texture));
        let u_sampler_location = gl
            .get_uniform_location(&self.shader, "uSampler")
            .expect("can't get uSampler location");
        gl.uniform1i(Some(&u_sampler_location), 0);

        gl.uniform_matrix4fv_with_f32_array(
            Some(
                &gl.get_uniform_location(&self.shader, "uProjectionMatrix")
                    .expect("can't get projection matrix location"),
            ),
            false,
            renderer.projection_matrix.as_matrix().as_slice(),
        );

        gl.uniform_matrix4fv_with_f32_array(
            Some(
                &gl.get_uniform_location(&self.shader, "uModelViewMatrix")
                    .expect("can't get model view matrix location"),
            ),
            false,
            renderer.model_view_matrix.as_slice(),
        );
    }

    pub fn new(shader: WebGlProgram, texture: WebGlTexture) -> Material {
        Material {
            shader,
            texture,
        }
    }
}