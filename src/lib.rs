extern crate core;

mod primitive;
mod mesh;
mod text;
mod drawable;
mod material;

use std::borrow::Borrow;
use crate::primitive::Primitive;
use crate::MouseState::{Down, Drag, Up};
use crate::ZoomState::{Idle, In, Out};
use gloo::render::{request_animation_frame, AnimationFrame};
use nalgebra::{Matrix4, Orthographic3, Point3, Vector2, Vector3};
use std::f32::consts::{PI, TAU};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Document, HtmlCanvasElement, HtmlImageElement, MouseEvent, WebGlProgram, WebGlRenderingContext,
    WebGlShader, WebGlTexture, WheelEvent, Window,
};
use crate::drawable::Drawable;
use crate::material::Material;

#[macro_export]
macro_rules! debug_web_number {
    ( $($div_name:expr, $value:expr)?) => {
        $(
        document()
            .get_element_by_id($div_name)
            .unwrap()
            .set_text_content(Some(format!("{:<5.1}", $value).as_str()));
        )?
    };
}

#[macro_export]
macro_rules! debug_web_vector2 {
    ( $($div_name:expr, $value:expr)?) => {
        $(
        document()
            .get_element_by_id($div_name)
            .unwrap()
            .set_text_content(Some(format!("{}, {}", $value.x, $value.y).as_str()));
        )?
    };
}

#[macro_export]
macro_rules! debug_web_vector3 {
    ( $($div_name:expr, $value:expr)?) => {
        $(
        document()
            .get_element_by_id($div_name)
            .unwrap()
            .set_text_content(Some(format!("{}, {}, {}", $value.x, $value.y, $value.z).as_str()));
        )?
    };
}

static mut RENDERER: Option<Renderer> = None;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(PartialEq)]
enum MouseState {
    Up,
    Down,
    Drag,
}

#[derive(PartialEq)]
enum ZoomState {
    In,
    Out,
    Idle,
}

pub struct Renderer {
    gl: WebGlRenderingContext,
    animation_handler: AnimationFrame,
    last_update: i32,
    display_width: i32,
    display_height: i32,
    aspect: f32,
    field_of_view: f32,
    z_near: f32,
    z_far: f32,
    camera_pos: Vector3<f32>,
    projection_matrix: Orthographic3<f32>,
    model_view_matrix: Matrix4<f32>,
    zoom: f32,
    zoom_state: ZoomState,
    entities: Vec<Primitive>,
    last_mouse_position: Vector2<f32>,
    current_mouse_position: Vector2<f32>,
    mouse_down_init_position: Vector2<f32>,
    mouse_drag_init_world_position: Vector3<f32>,
    mouse_state: MouseState,
    sample_delta: Vec<f32>,
}

#[wasm_bindgen]
pub fn run() {
    console_error_panic_hook::set_once();

    let canvas = canvas();

    let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
        let mouse_pos = get_mouse_position(event);
        let mut renderer = unsafe { RENDERER.as_mut().unwrap() };
        renderer.current_mouse_position = mouse_pos;
    });
    document()
        .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())
        .expect("failed to setup mousemove callback");
    closure.forget();

    let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
        let mut renderer = unsafe { RENDERER.as_mut().unwrap() };
        renderer.mouse_state = Down;
        renderer.mouse_down_init_position = get_mouse_position(event);
    });
    document()
        .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())
        .expect("failed to setup mousedown callback");
    closure.forget();

    let closure = Closure::<dyn FnMut(_)>::new(move |_event: MouseEvent| {
        let mut renderer = unsafe { RENDERER.as_mut().unwrap() };
        renderer.mouse_state = Up;
    });
    document()
        .add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())
        .expect("failed to setup mouseup callback");
    closure.forget();

    let closure = Closure::<dyn FnMut(_)>::new(move |event: WheelEvent| {
        let mut renderer = unsafe { RENDERER.as_mut().unwrap() };
        if event.delta_y() > 0.0 {
            renderer.zoom_state = In;
        } else {
            renderer.zoom_state = Out;
        }
    });
    document()
        .add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())
        .expect("failed to setup wheel callback");
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
    let material = Material::new(shader_program, texture);

    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear_depth(1.0);
    gl.enable(WebGlRenderingContext::DEPTH_TEST);
    gl.depth_func(WebGlRenderingContext::LEQUAL);
    gl.enable(WebGlRenderingContext::CULL_FACE);
    gl.pixel_storei(WebGlRenderingContext::UNPACK_FLIP_Y_WEBGL, 1);

    let mut entities: Vec<Primitive> = Vec::new();
    let factor = 3.464_101_6;
    for x in -6..7 {
        for y in -0..1 {
            let mut quad = Primitive::new_quad(&gl, material.clone());
            quad.position = Vector3::new(x as f32 * factor, y as f32 * factor, 0.0);
            entities.push(quad);
        }
    }

    unsafe {
        RENDERER = Some(Renderer {
            gl,
            animation_handler: request_animation_frame(update),
            last_update: 0,
            display_width: canvas.client_width(),
            display_height: canvas.client_height(),
            aspect: canvas.client_width() as f32 / canvas.client_height() as f32,
            field_of_view: 45.0 * PI / 180.0,
            z_near: 0.1,
            z_far: 100.0,
            camera_pos: Vector3::new(0.0, 0.0, 10.0),
            projection_matrix: Orthographic3::from_fov(1.0, 1.0, 0.0, 1.0), // dummy projection
            model_view_matrix: Matrix4::identity(), // another dummy
            zoom: 1.0,
            zoom_state: Idle,
            entities,
            last_mouse_position: Vector2::new(0.0, 0.0),
            current_mouse_position: Vector2::new(0.0, 0.0),
            mouse_down_init_position: Vector2::new(0.0, 0.0),
            mouse_drag_init_world_position: Vector3::new(0.0, 0.0, 0.0),
            mouse_state: Up,
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
    debug_web_number!["tps", avg_tps * 1000.0];

    renderer.last_mouse_position = renderer.current_mouse_position;
    update_mouse_state(renderer);

    let world_pos = get_world_pos_from_viewport_pos(renderer.last_mouse_position);

    debug_web_vector2!["mouse_world_position", world_pos];
    debug_web_vector3!["camera_position", renderer.camera_pos];

    // move camera such that mouse pos is still in the same world space position
    if Drag == renderer.mouse_state {
        let mut offset = world_pos - renderer.mouse_drag_init_world_position;
        offset.z = 0.0;
        renderer.camera_pos -= offset;
    }

    if Idle != renderer.zoom_state {
        match renderer.zoom_state {
            In => {
                renderer.zoom *= 1.1;
            }
            Out => {
                renderer.zoom *= 0.909_090_94;
            }
            Idle => {}
        }
        renderer.zoom = renderer.zoom.clamp(0.1, 3.7);
        renderer.zoom_state = Idle;
        renderer.projection_matrix = Orthographic3::from_fov(
            renderer.aspect,
            renderer.field_of_view * renderer.zoom,
            renderer.z_near,
            renderer.z_far,
        );
        let current_world_pos = get_world_pos_from_viewport_pos(renderer.last_mouse_position);
        let mut offset = world_pos - current_world_pos;
        offset.z = 0.0;
        renderer.camera_pos += offset;
    }

    for entity in &mut renderer.entities {
        let x_pos = entity.position.x;
        let x = entity.rotation.x;
        let y = entity.rotation.y;
        let z = entity.rotation.z;
        entity.rotation = Vector3::new(
            // 0.0,
            0.0,
            // (x + delta_time * x_pos * 0.1) % TAU,
            // (y + delta_time * x_pos * 0.1) % TAU,
            0.0,
            (z + delta_time * x_pos * 0.1) % TAU,
        );
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

fn draw_scene(renderer: &mut Renderer) {
    let gl = &renderer.gl;
    gl.viewport(0, 0, canvas().width() as i32, canvas().height() as i32);
    gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);

    renderer.aspect = renderer.display_width as f32 / renderer.display_height as f32;
    renderer.projection_matrix = Orthographic3::from_fov(
        renderer.aspect,
        renderer.field_of_view * renderer.zoom,
        renderer.z_near,
        renderer.z_far,
    );
    renderer.model_view_matrix = (Matrix4::new_translation(&renderer.camera_pos))
        .try_inverse()
        .unwrap();

    for entity in renderer.entities.as_slice() {
        entity.draw(&renderer);
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

fn update_mouse_state(renderer: &mut Renderer) {
    match renderer.mouse_state {
        Up => {}
        Down => {
            let mouse_delta = renderer.current_mouse_position - renderer.mouse_down_init_position;
            if mouse_delta.magnitude() > 2.0 {
                renderer.mouse_state = Drag;
                renderer.mouse_drag_init_world_position =
                    get_world_pos_from_viewport_pos(renderer.current_mouse_position);
            }
        }
        Drag => {}
    }
}

fn get_mouse_position(event: MouseEvent) -> Vector2<f32> {
    let canvas = canvas();
    let rect = canvas.get_bounding_client_rect();
    let scale_x = canvas.width() as f32 / rect.width() as f32;
    let scale_y = canvas.height() as f32 / rect.height() as f32;
    Vector2::new(
        (event.client_x() as f32 - rect.left() as f32) * scale_x,
        (event.client_y() as f32 - rect.top() as f32) * scale_y,
    )
}

fn get_world_pos_from_viewport_pos(pos: Vector2<f32>) -> Vector3<f32> {
    let renderer = unsafe { RENDERER.as_ref().unwrap() };
    let y_view = (pos.y - renderer.display_height as f32).abs();
    let x_clip = pos.x / renderer.display_width as f32 * 2.0 - 1.0;
    let y_clip = y_view / renderer.display_height as f32 * 2.0 - 1.0;
    let clip_space_pos = Point3::new(x_clip, y_clip, 1.0);
    let transformation_camera = Matrix4::new_translation(&renderer.camera_pos);
    let view_space_pos = renderer.projection_matrix.unproject_point(&clip_space_pos);
    let world_pos = Matrix4::new_translation(&view_space_pos.coords) * transformation_camera;
    Vector3::new(world_pos.m14, world_pos.m24, world_pos.m34)
}

fn is_power_of_2(value: u32) -> bool {
    value & (value - 1) == 0
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
