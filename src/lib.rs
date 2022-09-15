extern crate core;

mod entity;
mod mesh;

use crate::entity::Entity;
use gloo::render::{request_animation_frame, AnimationFrame};
use nalgebra::{Matrix4, Orthographic3, Vector2, Vector3};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, MouseEvent, HtmlCanvasElement, HtmlImageElement, WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlTexture, Window};

static mut RENDERER: Option<Renderer> = None;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

enum MouseState {
    Idle,
    Down,
    Pressed,
}

struct Renderer {
    gl: WebGlRenderingContext,
    program: WebGlProgram,
    animation_handler: AnimationFrame,
    texture: WebGlTexture,
    last_update: i32,
    display_width: i32,
    display_height: i32,
    camera_pos: Vector3<f32>,
    camera_rot: Vector3<f32>,
    entities: Vec<Entity>,
    last_mouse_position: Vector2<f64>,
    current_mouse_position: Vector2<f64>,
    mouse_state: MouseState,
    sample_delta: Vec<f32>,
}

#[wasm_bindgen]
pub fn run() {
    console_error_panic_hook::set_once();

    let canvas = canvas();

    let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
        let mouse_pos = get_mouse_position(event);
        document()
            .get_element_by_id("mouse_pos")
            .unwrap()
            .set_text_content(Some(format!("{}, {}", mouse_pos.x, mouse_pos.y).as_str()));
        let mut renderer = unsafe { RENDERER.as_mut().unwrap() };
        renderer.current_mouse_position = mouse_pos;
    });
    document().add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref()).expect("failed to setup mousemove callback");
    closure.forget();

    let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
        let mut renderer = unsafe { RENDERER.as_mut().unwrap() };
        renderer.mouse_state = MouseState::Down;
    });
    document().add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref()).expect("failed to setup mousemove callback");
    closure.forget();

    let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
        let mut renderer = unsafe { RENDERER.as_mut().unwrap() };
        renderer.mouse_state = MouseState::Idle;
    });
    document().add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref()).expect("failed to setup mousemove callback");
    closure.forget();

    let gl = canvas
        .get_context("webgl")
        .expect("unable to initialize WebGL, your browser or machine may not support it.")
        .expect("failed to retrieve context")
        .dyn_into::<WebGlRenderingContext>()
        .expect("failed to convert context into webgl context");

    let vertex_shader_source = include_str!("vs.glsl");
    let fragment_shader_source = include_str!("fs.glsl");
    let shader_program = init_shader_program(&gl, vertex_shader_source, fragment_shader_source)
        .expect("failed to create shader program");

    let texture = load_texture(&gl, "http://localhost:8000/texture/rust_logo.png");

    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear_depth(1.0);
    gl.enable(WebGlRenderingContext::DEPTH_TEST);
    gl.depth_func(WebGlRenderingContext::LEQUAL);
    gl.enable(WebGlRenderingContext::CULL_FACE);
    gl.pixel_storei(WebGlRenderingContext::UNPACK_FLIP_Y_WEBGL, 1);

    let mut entities: Vec<Entity> = Vec::new();
    let factor = 3.464_101_6;
    for x in -4..4 {
        for y in -1..2 {
            let mut cube = Entity::new(&gl);
            cube.position =
                Vector3::new(x as f32 * factor, y as f32 * factor, 0.0);
            entities.push(cube);
        }
    }

    unsafe {
        RENDERER = Some(Renderer {
            gl,
            program: shader_program,
            animation_handler: request_animation_frame(update),
            last_update: 0,
            display_width: canvas.client_width(),
            display_height: canvas.client_height(),
            camera_pos: Vector3::new(0.0, 0.0, 10.0),
            camera_rot: Vector3::new(0.0, 0.0, 0.0),
            texture,
            entities,
            last_mouse_position: Vector2::new(0.0, 0.0),
            current_mouse_position: Vector2::new(0.0, 0.0),
            mouse_state: MouseState::Idle,
            sample_delta: Vec::new(),
        })
    }
}

fn update(timestamp: f64) {
    let timestamp = timestamp as i32;
    let mut renderer = unsafe { RENDERER.as_mut().unwrap() };
    let delta_time = ((timestamp - renderer.last_update) as f32) * 0.001;
    renderer.last_update = timestamp;

    renderer.sample_delta.push(delta_time);
    if renderer.sample_delta.len() >= 10 {
        renderer.sample_delta.remove(0);
    }
    let avg_tps: f32 = renderer.sample_delta.iter().sum();
    let avg_tps = avg_tps / renderer.sample_delta.len() as f32;
    document()
        .get_element_by_id("tps")
        .unwrap()
        .set_text_content(Some(format!("{:<5.1}", avg_tps * 1000.0).as_str()));

    let mouse_delta = renderer.current_mouse_position - renderer.last_mouse_position;
    renderer.last_mouse_position = renderer.current_mouse_position;
    document()
        .get_element_by_id("mouse_delta")
        .unwrap()
        .set_text_content(Some(format!("{}, {}", mouse_delta.x, mouse_delta.y).as_str()));


    for entity in &mut renderer.entities {
        let x_pos = entity.position.x;
        let x = entity.rotation.x;
        let y = entity.rotation.x;
        entity.rotation = Vector3::new(x + delta_time + x_pos * 0.001, y + delta_time, 0.0);
    }

    if let MouseState::Down = renderer.mouse_state {
        renderer.camera_pos.x -= mouse_delta.x as f32 * 0.01;
        renderer.camera_pos.y += mouse_delta.y as f32 * 0.01;
    }

    let canvas = canvas(); // todo: only work on chrome ???
    renderer.display_width = canvas.client_width();
    renderer.display_height = canvas.client_height();
    // set draw buffer size to display size
    canvas.set_width(renderer.display_width as u32);
    canvas.set_height(renderer.display_height as u32);

    draw_scene(renderer);

    renderer.animation_handler = request_animation_frame(update);
}

fn draw_scene(renderer: &Renderer) {
    let gl = &renderer.gl;
    gl.viewport(0, 0, canvas().width() as i32, canvas().height() as i32);
    gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);

    let aspect = renderer.display_width as f32 / renderer.display_height as f32;
    let field_of_view = 45.0 * std::f32::consts::PI / 180.0;
    let z_near = 0.1;
    let z_far = 100.0;
    let projection_matrix = Orthographic3::from_fov(aspect, field_of_view, z_near, z_far);
    let model_view_matrix = (Matrix4::new_translation(&renderer.camera_pos)
        * Matrix4::from_euler_angles(
            renderer.camera_rot.x,
            renderer.camera_rot.y,
            renderer.camera_rot.z,
        ))
    .try_inverse()
    .unwrap();

    gl.use_program(Some(&renderer.program));

    gl.uniform_matrix4fv_with_f32_array(
        Some(
            &gl.get_uniform_location(&renderer.program, "uProjectionMatrix")
                .expect("can't get projection matrix location"),
        ),
        false,
        projection_matrix.as_matrix().as_slice(),
    );

    gl.uniform_matrix4fv_with_f32_array(
        Some(
            &gl.get_uniform_location(&renderer.program, "uModelViewMatrix")
                .expect("can't get model view matrix location"),
        ),
        false,
        model_view_matrix.as_slice(),
    );

    gl.active_texture(WebGlRenderingContext::TEXTURE0);
    gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&renderer.texture));
    let u_sampler_location = gl
        .get_uniform_location(&renderer.program, "uSampler")
        .expect("can't get uSampler location");
    gl.uniform1i(Some(&u_sampler_location), 0);

    for entity in renderer.entities.as_slice() {
        entity.draw(&renderer.gl, &renderer.program);
    }
}

fn init_shader_program(
    gl: &WebGlRenderingContext,
    vss: &str,
    fss: &str,
) -> Result<WebGlProgram, ()> {
    let vertex_shader = load_shader(gl, WebGlRenderingContext::VERTEX_SHADER, vss)
        .expect("failed to load vertex shader");
    let fragment_shader = load_shader(gl, WebGlRenderingContext::FRAGMENT_SHADER, fss)
        .expect("failed to load fragment shader");

    let shader_program = gl
        .create_program()
        .expect("failed to create shader program");
    gl.attach_shader(&shader_program, &vertex_shader);
    gl.attach_shader(&shader_program, &fragment_shader);
    gl.link_program(&shader_program);

    if !gl.get_program_parameter(&shader_program, WebGlRenderingContext::LINK_STATUS) {
        let _info_log = gl
            .get_program_info_log(&shader_program)
            .expect("can't retrieve information log");
        alert(format!("an error occurred linking the shaders: {_info_log}").as_str());
        return Err(());
    }

    Ok(shader_program)
}

fn load_shader(gl: &WebGlRenderingContext, type_: u32, source: &str) -> Result<WebGlShader, ()> {
    let shader = gl.create_shader(type_).expect("failed to create shader");
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    if !gl.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS) {
        let _info_log = gl
            .get_shader_info_log(&shader)
            .expect("can't retrieve information log");
        alert(format!("an error occurred compiling the shaders: {_info_log}").as_str());
        gl.delete_shader(Some(&shader));
        return Err(());
    }
    Ok(shader)
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

    image.set_onload(Some(callback.as_ref().unchecked_ref()));
    image.set_cross_origin(Some("anonymous"));
    image.set_src(path);

    callback.forget();

    texture
}

fn is_power_of_2(value: u32) -> bool {
    value & (value - 1) == 0
}

fn get_mouse_position(event: MouseEvent) -> Vector2<f64>{
    let canvas = canvas();
    let rect = canvas.get_bounding_client_rect();
    let scale_x = canvas.width() as f64 / rect.width();
    let scale_y = canvas.height() as f64 / rect.height();
    Vector2::new((event.client_x() as f64 - rect.left()) * scale_x, (event.client_y() as f64 - rect.top()) * scale_y)
}

fn window() -> Window {
    web_sys::window().expect("no global `window` exists")
}

fn document() -> Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn canvas() -> HtmlCanvasElement {
    document()
        .get_element_by_id("canvas")
        .expect("failed to get #canvas element")
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .expect("failed to get html canvas element")
}
