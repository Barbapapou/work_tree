use gloo::render::{request_animation_frame, AnimationFrame};
use nalgebra::{Matrix4, Unit, Vector3};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    HtmlImageElement, WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlTexture,
    Window,
};

static mut RENDERER: Option<Renderer> = None;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

struct Renderer {
    gl: WebGlRenderingContext,
    program: WebGlProgram,
    vbo: VertexBufferObject,
    animation_handler: AnimationFrame,
    texture: WebGlTexture,
    cube_rotation: f32,
    last_update: i32,
}

struct VertexBufferObject {
    position: WebGlBuffer,
    uv: WebGlBuffer,
    indices: WebGlBuffer,
}

#[wasm_bindgen]
pub fn run() {
    console_error_panic_hook::set_once();

    let document = window()
        .document()
        .expect("should have a document on window");
    let canvas = document
        .get_element_by_id("canvas")
        .expect("failed to get #canvas element");
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .expect("failed to get html canvas element");
    let gl = canvas
        .get_context("webgl")
        .expect("unable to initialize WebGL, your browser or machine may not support it.")
        .expect("failed to retrieve context")
        .dyn_into::<web_sys::WebGlRenderingContext>()
        .expect("failed to convert context into webgl context");

    let vertex_shader_source = include_str!("vs.glsl");
    let fragment_shader_source = include_str!("fs.glsl");
    let shader_program = init_shader_program(&gl, vertex_shader_source, fragment_shader_source)
        .expect("failed to create shader program");
    let vbo = init_buffer(&gl);
    let texture = load_texture(&gl, "http://localhost:8000/texture/rust_logo.png");
    gl.pixel_storei(WebGlRenderingContext::UNPACK_FLIP_Y_WEBGL, 1); // todo check if 1 == true

    unsafe {
        RENDERER = Some(Renderer {
            gl,
            program: shader_program,
            vbo,
            animation_handler: request_animation_frame(update),
            cube_rotation: 0.0,
            last_update: 0,
            texture,
        })
    }
}

fn update(timestamp: f64) {
    let timestamp = timestamp as i32;
    let renderer = unsafe { RENDERER.as_mut().unwrap() };
    let delta_time = ((timestamp - renderer.last_update) as f32) * 0.001;
    renderer.cube_rotation += delta_time;
    renderer.last_update = timestamp;
    draw_scene(&renderer);
    renderer.animation_handler = request_animation_frame(update);
}

fn draw_scene(renderer: &Renderer) {
    renderer.gl.clear_color(0.0, 0.0, 0.0, 1.0);
    renderer.gl.clear_depth(1.0);
    renderer.gl.enable(WebGlRenderingContext::DEPTH_TEST);
    renderer.gl.depth_func(WebGlRenderingContext::LEQUAL);

    renderer
        .gl
        .clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);

    let aspect = 16.0 / 9.0;
    let field_of_view = 45.0 * std::f32::consts::PI / 180.0;
    let z_near = 0.1;
    let z_far = 100.0;
    let projection_matrix = Matrix4::new_perspective(aspect, field_of_view, z_near, z_far);
    let model_view_matrix = Matrix4::new_translation(&Vector3::new(-0.0, 0.0, -6.0));
    let model_view_matrix = model_view_matrix
        * Matrix4::from_axis_angle(
            Unit::from_ref_unchecked(&Vector3::new(0.0, 0.0, 1.0)),
            renderer.cube_rotation,
        )
        * Matrix4::from_axis_angle(
            Unit::from_ref_unchecked(&Vector3::new(0.0, 1.0, 0.0)),
            renderer.cube_rotation * 0.7,
        )
        * Matrix4::from_axis_angle(
            Unit::from_ref_unchecked(&Vector3::new(1.0, 0.0, 0.0)),
            renderer.cube_rotation * 0.4,
        );

    // Position
    let num_components = 3;
    let type_ = WebGlRenderingContext::FLOAT;
    let normalize = false;
    let stride = 0;
    let offset = 0;

    // get the location of the aVertexPosition shader param
    let attrib_vertex_position = renderer
        .gl
        .get_attrib_location(&renderer.program, "aVertexPosition")
        as u32;

    renderer.gl.bind_buffer(
        WebGlRenderingContext::ARRAY_BUFFER,
        Some(&renderer.vbo.position),
    );
    renderer.gl.vertex_attrib_pointer_with_i32(
        attrib_vertex_position,
        num_components,
        type_,
        normalize,
        stride,
        offset,
    );
    renderer
        .gl
        .enable_vertex_attrib_array(attrib_vertex_position);

    // Uv
    let num_components = 2;
    let type_ = WebGlRenderingContext::FLOAT;
    let normalize = false;
    let stride = 0;
    let offset = 0;

    let attrib_uv = renderer
        .gl
        .get_attrib_location(&renderer.program, "aTextureCoord") as u32;

    renderer
        .gl
        .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&renderer.vbo.uv));
    renderer.gl.vertex_attrib_pointer_with_i32(
        attrib_uv,
        num_components,
        type_,
        normalize,
        stride,
        offset,
    );
    renderer.gl.enable_vertex_attrib_array(attrib_uv);

    renderer.gl.bind_buffer(
        WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
        Some(&renderer.vbo.indices),
    );

    renderer.gl.use_program(Some(&renderer.program));

    renderer.gl.uniform_matrix4fv_with_f32_array(
        Some(
            &renderer
                .gl
                .get_uniform_location(&renderer.program, "uProjectionMatrix")
                .expect("can't get projection matrix location"),
        ),
        false,
        projection_matrix.as_slice(),
    );

    renderer.gl.uniform_matrix4fv_with_f32_array(
        Some(
            &renderer
                .gl
                .get_uniform_location(&renderer.program, "uModelViewMatrix")
                .expect("can't get model view matrix location"),
        ),
        false,
        model_view_matrix.as_slice(),
    );

    renderer.gl.active_texture(WebGlRenderingContext::TEXTURE0);
    renderer
        .gl
        .bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&renderer.texture));
    let u_sampler_location = renderer
        .gl
        .get_uniform_location(&renderer.program, "uSampler")
        .expect("can't get uSampler location");
    renderer.gl.uniform1i(Some(&u_sampler_location), 0);

    let offset = 0;
    let vertex_count = 36;
    let type_ = WebGlRenderingContext::UNSIGNED_SHORT;
    renderer.gl.draw_elements_with_i32(
        WebGlRenderingContext::TRIANGLES,
        vertex_count,
        type_,
        offset,
    );
}

fn init_shader_program(
    gl: &WebGlRenderingContext,
    vss: &str,
    fss: &str,
) -> Result<WebGlProgram, ()> {
    let vertex_shader = load_shader(&gl, WebGlRenderingContext::VERTEX_SHADER, vss)
        .expect("failed to load vertex shader");
    let fragment_shader = load_shader(&gl, WebGlRenderingContext::FRAGMENT_SHADER, fss)
        .expect("failed to load fragment shader");

    let shader_program = gl
        .create_program()
        .expect("failed to create shader program");
    gl.attach_shader(&shader_program, &vertex_shader);
    gl.attach_shader(&shader_program, &fragment_shader);
    gl.link_program(&shader_program);

    if !gl.get_program_parameter(&shader_program, WebGlRenderingContext::LINK_STATUS) {
        let info_log = gl
            .get_program_info_log(&shader_program)
            .expect("can't retrieve information log");
        alert(format!("an error occurred linking the shaders: {info_log}").as_str());
        return Err(());
    }

    Ok(shader_program)
}

fn load_shader(gl: &WebGlRenderingContext, type_: u32, source: &str) -> Result<WebGlShader, ()> {
    let shader = gl.create_shader(type_).expect("failed to create shader");
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    if !gl.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS) {
        let info_log = gl
            .get_shader_info_log(&shader)
            .expect("can't retrieve information log");
        alert(format!("an error occurred compiling the shaders: {info_log}").as_str());
        gl.delete_shader(Some(&shader));
        return Err(());
    }
    Ok(shader)
}

fn init_buffer(gl: &WebGlRenderingContext) -> VertexBufferObject {
    let position_buffer = gl.create_buffer().expect("failed to create buffer");
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&position_buffer));
    #[rustfmt::skip]
    let vertices = [
        // Front face
        -1.0, -1.0,  1.0,
        1.0, -1.0,  1.0,
        1.0,  1.0,  1.0,
        -1.0,  1.0,  1.0,

        // Back face
        -1.0, -1.0, -1.0,
        -1.0,  1.0, -1.0,
        1.0,  1.0, -1.0,
        1.0, -1.0, -1.0,

        // Top face
        -1.0,  1.0, -1.0,
        -1.0,  1.0,  1.0,
        1.0,  1.0,  1.0,
        1.0,  1.0, -1.0,

        // Bottom face
        -1.0, -1.0, -1.0,
        1.0, -1.0, -1.0,
        1.0, -1.0,  1.0,
        -1.0, -1.0,  1.0,

        // Right face
        1.0, -1.0, -1.0,
        1.0,  1.0, -1.0,
        1.0,  1.0,  1.0,
        1.0, -1.0,  1.0,

        // Left face
        -1.0, -1.0, -1.0,
        -1.0, -1.0,  1.0,
        -1.0,  1.0,  1.0,
        -1.0,  1.0, -1.0,
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
        0.0,  0.0,
        1.0,  0.0,
        1.0,  1.0,
        0.0,  1.0,
        // Back
        0.0,  0.0,
        1.0,  0.0,
        1.0,  1.0,
        0.0,  1.0,
        // Top
        0.0,  0.0,
        1.0,  0.0,
        1.0,  1.0,
        0.0,  1.0,
        // Bottom
        0.0,  0.0,
        1.0,  0.0,
        1.0,  1.0,
        0.0,  1.0,
        // Right
        0.0,  0.0,
        1.0,  0.0,
        1.0,  1.0,
        0.0,  1.0,
        // Left
        0.0,  0.0,
        1.0,  0.0,
        1.0,  1.0,
        0.0,  1.0,
    ];
    unsafe {
        let uvs = js_sys::Float32Array::view(&uvs);
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &(uvs),
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
        0,  1,  2,      0,  2,  3,    // front
        4,  5,  6,      4,  6,  7,    // back
        8,  9,  10,     8,  10, 11,   // top
        12, 13, 14,     12, 14, 15,   // bottom
        16, 17, 18,     16, 18, 19,   // right
        20, 21, 22,     20, 22, 23,   // left
    ];
    unsafe {
        let indices = js_sys::Uint16Array::view(&indices);
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            &(indices),
            WebGlRenderingContext::STATIC_DRAW,
        );
    }

    VertexBufferObject {
        position: position_buffer,
        uv: uv_buffer,
        indices: index_buffer,
    }
}

fn load_texture(gl: &WebGlRenderingContext, path: &str) -> WebGlTexture {
    let texture = gl.create_texture().expect("failed to create texture");
    gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture));

    let level = 0;
    let internal_format = WebGlRenderingContext::RGBA;
    let width = 1;
    let height = 1;
    let border = 0;
    let source_format = WebGlRenderingContext::RGBA;
    let source_type = WebGlRenderingContext::UNSIGNED_BYTE;
    let pixel = [255, 0, 255, 255];
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        WebGlRenderingContext::TEXTURE_2D,
        level,
        internal_format as i32,
        width,
        height,
        border,
        source_format,
        source_type,
        Some(&pixel),
    )
    .expect("failed to copy dummy texture data");

    let image = HtmlImageElement::new().expect("failed to create new image");

    let callback = Closure::<dyn Fn()>::new({
        let gl = gl.clone();
        let texture = texture.clone();
        let image = image.clone();
        move || {
            gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture));
            gl.tex_image_2d_with_u32_and_u32_and_image(
                WebGlRenderingContext::TEXTURE_2D,
                0,
                WebGlRenderingContext::RGBA as i32,
                WebGlRenderingContext::RGBA,
                WebGlRenderingContext::UNSIGNED_BYTE,
                &image,
            )
            .expect("failed to load image texture");

            if is_power_of_2(image.width()) && is_power_of_2(image.height()) {
                gl.generate_mipmap(WebGlRenderingContext::TEXTURE_2D);
            } else {
                gl.tex_parameteri(
                    WebGlRenderingContext::TEXTURE_2D,
                    WebGlRenderingContext::TEXTURE_MIN_FILTER,
                    WebGlRenderingContext::LINEAR as i32,
                );
                gl.tex_parameteri(
                    WebGlRenderingContext::TEXTURE_2D,
                    WebGlRenderingContext::TEXTURE_WRAP_S,
                    WebGlRenderingContext::CLAMP_TO_EDGE as i32,
                );
                gl.tex_parameteri(
                    WebGlRenderingContext::TEXTURE_2D,
                    WebGlRenderingContext::TEXTURE_WRAP_T,
                    WebGlRenderingContext::CLAMP_TO_EDGE as i32,
                );
            }
        }
    });

    image.set_onload(Some(&callback.as_ref().unchecked_ref()));
    image.set_cross_origin(Some("anonymous"));
    image.set_src(path);

    callback.forget();

    texture
}

fn is_power_of_2(value: u32) -> bool {
    return value & (value - 1) == 0;
}

fn window() -> Window {
    web_sys::window().expect("no global `window` exists")
}
