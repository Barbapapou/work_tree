use crate::mesh::Mesh;
use nalgebra::{Matrix4, Vector3};
use web_sys::{WebGlProgram, WebGlRenderingContext};

pub struct Entity {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Vector3<f32>,
    mesh: Mesh,
}

impl Entity {
    fn bind(&self, gl: &WebGlRenderingContext, shader: &WebGlProgram) {
        self.mesh.bind(gl, shader);
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

    pub fn draw(&self, gl: &WebGlRenderingContext, shader: &WebGlProgram) {
        self.bind(gl, shader);

        let offset = 0;
        let vertex_count = 36;
        let type_ = WebGlRenderingContext::UNSIGNED_SHORT;
        gl.draw_elements_with_i32(
            WebGlRenderingContext::TRIANGLES,
            vertex_count,
            type_,
            offset,
        );
    }

    pub fn new(gl: &WebGlRenderingContext) -> Entity {
        Entity {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
            mesh: Mesh::cube(gl),
        }
    }
}
