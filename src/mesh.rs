use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext};

struct VertexBufferObject {
    position: WebGlBuffer,
    uv: WebGlBuffer,
    normal: WebGlBuffer,
    indices: WebGlBuffer,
}

pub struct Mesh {
    vbo: VertexBufferObject,
}

impl Mesh {
    pub fn bind(&self, gl: &WebGlRenderingContext, shader: &WebGlProgram) {
        // Position
        // get the location of the aVertexPosition shader param
        let attrib_vertex_position = gl.get_attrib_location(shader, "aVertexPosition") as u32;
        gl.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.vbo.position),
        );
        gl.vertex_attrib_pointer_with_i32(
            attrib_vertex_position,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        gl.enable_vertex_attrib_array(attrib_vertex_position);

        // Uv
        let attrib_uv = gl.get_attrib_location(shader, "aTextureCoord") as u32;
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.vbo.uv));
        gl.vertex_attrib_pointer_with_i32(attrib_uv, 2, WebGlRenderingContext::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(attrib_uv);

        // Normal
        let attrib_normal = gl.get_attrib_location(shader, "aVertexNormal") as u32;
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.vbo.normal));
        gl.vertex_attrib_pointer_with_i32(
            attrib_normal,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        gl.enable_vertex_attrib_array(attrib_normal);

        gl.bind_buffer(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&self.vbo.indices),
        );
    }

    pub fn cube(gl: &WebGlRenderingContext) -> Mesh {
        let position_buffer = gl.create_buffer().expect("failed to create buffer");
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&position_buffer));
        #[rustfmt::skip]
            let vertices = [
            // Front face
            -1.0, -1.0, 1.0,
            1.0, -1.0, 1.0,
            1.0, 1.0, 1.0,
            -1.0, 1.0, 1.0,

            // Back face
            -1.0, -1.0, -1.0,
            -1.0, 1.0, -1.0,
            1.0, 1.0, -1.0,
            1.0, -1.0, -1.0,

            // Top face
            -1.0, 1.0, -1.0,
            -1.0, 1.0, 1.0,
            1.0, 1.0, 1.0,
            1.0, 1.0, -1.0,

            // Bottom face
            -1.0, -1.0, -1.0,
            1.0, -1.0, -1.0,
            1.0, -1.0, 1.0,
            -1.0, -1.0, 1.0,

            // Right face
            1.0, -1.0, -1.0,
            1.0, 1.0, -1.0,
            1.0, 1.0, 1.0,
            1.0, -1.0, 1.0,

            // Left face
            -1.0, -1.0, -1.0,
            -1.0, -1.0, 1.0,
            -1.0, 1.0, 1.0,
            -1.0, 1.0, -1.0,
        ];
        unsafe {
            let vertices = js_sys::Float32Array::view(&vertices);
            gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &(vertices),
                WebGlRenderingContext::STATIC_DRAW,
            );
        }

        let uv_buffer = gl.create_buffer().expect("failed to create buffer");
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&uv_buffer));
        #[rustfmt::skip]
            let uvs = [
            // Front
            0.0, 0.0,
            1.0, 0.0,
            1.0, 1.0,
            0.0, 1.0,
            // Back
            0.0, 0.0,
            1.0, 0.0,
            1.0, 1.0,
            0.0, 1.0,
            // Top
            0.0, 0.0,
            1.0, 0.0,
            1.0, 1.0,
            0.0, 1.0,
            // Bottom
            0.0, 0.0,
            1.0, 0.0,
            1.0, 1.0,
            0.0, 1.0,
            // Right
            0.0, 0.0,
            1.0, 0.0,
            1.0, 1.0,
            0.0, 1.0,
            // Left
            0.0, 0.0,
            1.0, 0.0,
            1.0, 1.0,
            0.0, 1.0,
        ];
        unsafe {
            let uvs = js_sys::Float32Array::view(&uvs);
            gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &(uvs),
                WebGlRenderingContext::STATIC_DRAW,
            );
        }

        let normal_buffer = gl.create_buffer().expect("failed to create buffer");
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&normal_buffer));
        #[rustfmt::skip]
            let normals = [
            // Front
            0.0, 0.0, 1.0,
            0.0, 0.0, 1.0,
            0.0, 0.0, 1.0,
            0.0, 0.0, 1.0,

            // Back
            0.0, 0.0, -1.0,
            0.0, 0.0, -1.0,
            0.0, 0.0, -1.0,
            0.0, 0.0, -1.0,

            // Top
            0.0, 1.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 1.0, 0.0,

            // Bottom
            0.0, -1.0, 0.0,
            0.0, -1.0, 0.0,
            0.0, -1.0, 0.0,
            0.0, -1.0, 0.0,

            // Right
            1.0, 0.0, 0.0,
            1.0, 0.0, 0.0,
            1.0, 0.0, 0.0,
            1.0, 0.0, 0.0,

            // Left
            -1.0, 0.0, 0.0,
            -1.0, 0.0, 0.0,
            -1.0, 0.0, 0.0,
            -1.0, 0.0, 0.0
        ];
        unsafe {
            let normals = js_sys::Float32Array::view(&normals);
            gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &(normals),
                WebGlRenderingContext::STATIC_DRAW,
            );
        }

        let index_buffer = gl.create_buffer().expect("failed to create buffer");
        gl.bind_buffer(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&index_buffer),
        );
        #[rustfmt::skip]
            let indices = [
            0, 1, 2, 0, 2, 3,    // front
            4, 5, 6, 4, 6, 7,    // back
            8, 9, 10, 8, 10, 11,   // top
            12, 13, 14, 12, 14, 15,   // bottom
            16, 17, 18, 16, 18, 19,   // right
            20, 21, 22, 20, 22, 23,   // left
        ];
        unsafe {
            let indices = js_sys::Uint16Array::view(&indices);
            gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
                &(indices),
                WebGlRenderingContext::STATIC_DRAW,
            );
        }

        let vbo = VertexBufferObject {
            position: position_buffer,
            uv: uv_buffer,
            normal: normal_buffer,
            indices: index_buffer,
        };

        Mesh { vbo }
    }
}
