use crate::mesh::Mesh;
use nalgebra::{Matrix4, Vector3};
use web_sys::{WebGlRenderingContext};
use crate::drawable::Drawable;
use crate::material::Material;
use crate::Renderer;

pub struct Primitive {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Vector3<f32>,
    mesh: Mesh,
    material: Material,
}

impl Drawable for Primitive {
    fn draw(&self, renderer: &Renderer) {
        let gl = &renderer.gl;
        self.bind(renderer);
        gl.draw_elements_with_i32(
            WebGlRenderingContext::TRIANGLES,
            self.mesh.vertex_count,
            WebGlRenderingContext::UNSIGNED_SHORT,
            0,
        );
    }
}

impl Primitive {
    fn bind(&self, renderer: &Renderer) {
        let gl = &renderer.gl;
        let shader = &self.material.shader;
        self.mesh.bind(gl, &self.material);
        self.material.bind(gl);

        gl.uniform_matrix4fv_with_f32_array(
            Some(
                &gl.get_uniform_location(shader, "uProjectionMatrix")
                    .expect("can't get projection matrix location"),
            ),
            false,
            renderer.projection_matrix.as_matrix().as_slice(),
        );

        gl.uniform_matrix4fv_with_f32_array(
            Some(
                &gl.get_uniform_location(shader, "uModelViewMatrix")
                    .expect("can't get model view matrix location"),
            ),
            false,
            renderer.model_view_matrix.as_slice(),
        );

        // add transformation uniform
        let transformation_matrix = Matrix4::new_translation(&self.position)
            * Matrix4::from_euler_angles(self.rotation.x, self.rotation.y, self.rotation.z)
            * Matrix4::new_nonuniform_scaling(&self.scale);

        gl.uniform_matrix4fv_with_f32_array(
            Some(
                &gl.get_uniform_location(shader, "uTransformationMatrix")
                    .expect("can't get transformation matrix location"),
            ),
            false,
            transformation_matrix.as_slice(),
        );

        let normal_matrix = transformation_matrix
            .try_inverse()
            .expect("failed to inverse model view matrix for normal matrix creation")
            .transpose();

        gl.uniform_matrix4fv_with_f32_array(
            Some(
                &gl.get_uniform_location(shader, "uNormalMatrix")
                    .expect("can't get normal matrix location"),
            ),
            false,
            normal_matrix.as_slice(),
        );
    }

    pub fn new_quad(gl: &WebGlRenderingContext, material: Material) -> Primitive {
        Primitive {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
            mesh: Mesh::quad(gl),
            material,
        }
    }

    pub fn new_cube(gl: &WebGlRenderingContext, material: Material) -> Primitive {
        Primitive {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
            mesh: Mesh::cube(gl),
            material
        }
    }
}
