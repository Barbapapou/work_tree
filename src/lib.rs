use nalgebra::{Matrix4, Vector3};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader};

struct VertexBufferObject {
    position: WebGlBuffer,
    color: WebGlBuffer,
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
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
    draw_scene(&gl, &shader_program, &vbo);

    Ok(())
}

fn draw_scene(gl: &WebGlRenderingContext, program_info: &WebGlProgram, vbo: &VertexBufferObject) {
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear_depth(1.0);
    gl.enable(WebGlRenderingContext::DEPTH_TEST);
    gl.depth_func(WebGlRenderingContext::LEQUAL);

    gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);

    let aspect = 16.0 / 9.0;
    let field_of_view = 45.0 * std::f32::consts::PI / 180.0;
    let z_near = 0.1;
    let z_far = 100.0;
    let projection_matrix = Matrix4::new_perspective(aspect, field_of_view, z_near, z_far);
    let model_view_matrix = Matrix4::new_translation(&Vector3::new(-0.0, 0.0, -6.0));

    // Position
    let num_components = 2;
    let type_ = WebGlRenderingContext::FLOAT;
    let normalize = false;
    let stride = 0;
    let offset = 0;

    // get the location of the aVertexPosition shader param
    let attrib_vertex_position = gl.get_attrib_location(&program_info, "aVertexPosition") as u32;

    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vbo.position));
    gl.vertex_attrib_pointer_with_i32(
        attrib_vertex_position,
        num_components,
        type_,
        normalize,
        stride,
        offset,
    );
    gl.enable_vertex_attrib_array(attrib_vertex_position);

    // Color
    let num_components = 4;
    let type_ = WebGlRenderingContext::FLOAT;
    let normalize = false;
    let stride = 0;
    let offset = 0;

    let attrib_vertex_color = gl.get_attrib_location(&program_info, "aVertexColor") as u32;

    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vbo.color));
    gl.vertex_attrib_pointer_with_i32(
        attrib_vertex_color,
        num_components,
        type_,
        normalize,
        stride,
        offset,
    );
    gl.enable_vertex_attrib_array(attrib_vertex_color);

    gl.use_program(Some(&program_info));

    gl.uniform_matrix4fv_with_f32_array(
        Some(
            &gl.get_uniform_location(&program_info, "uProjectionMatrix")
                .expect("can't get projection matrix location"),
        ),
        false,
        projection_matrix.as_slice(),
    );

    gl.uniform_matrix4fv_with_f32_array(
        Some(
            &gl.get_uniform_location(&program_info, "uModelViewMatrix")
                .expect("can't get model view matrix location"),
        ),
        false,
        model_view_matrix.as_slice(),
    );

    let offset = 0;
    let vertex_count = 4;
    gl.draw_arrays(WebGlRenderingContext::TRIANGLE_STRIP, offset, vertex_count);
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
        1.0 ,  1.0,
        -1.0,  1.0,
        1.0 , -1.0,
        -1.0, -1.0
    ];
    unsafe {
        let vertices = js_sys::Float32Array::view(&vertices);
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &(vertices),
            WebGlRenderingContext::STATIC_DRAW,
        );
    }

    let color_buffer = gl.create_buffer().expect("failed to create buffer");
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&color_buffer));
    #[rustfmt::skip]
    let colors = [
        1.0, 1.0, 1.0, 1.0,
        1.0, 0.0, 0.0, 1.0,
        0.0, 1.0, 0.0, 1.0,
        0.0, 0.0, 1.0, 1.0,
    ];
    unsafe {
        let colors = js_sys::Float32Array::view(&colors);
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &(colors),
            WebGlRenderingContext::STATIC_DRAW,
        );
    }

    VertexBufferObject {
        position: position_buffer,
        color: color_buffer,
    }
}
