// use crate::Primitive;
// use web_sys::{WebGlProgram, WebGlRenderingContext};
// 
// pub struct Text {
//     geometry: Vec<Primitive>,
// }
// 
// impl Text {
//     pub fn new(input: &str, gl: &WebGlRenderingContext) -> Text {
//         let mut geometry = Vec::new();
//         for (i, char) in input.chars().enumerate() {
//             if char != ' ' {
//                 // let entity = Entity::new_glyph(i as f32, gl);
//                 // geometry.push(entity);
//             }
//         }
//         Text { geometry }
//     }
// 
//     pub fn draw(&self, shader: &WebGlProgram, gl: &WebGlRenderingContext) {
//         // bind text texture
//         for entity in &self.geometry {
//             entity.draw(gl, shader);
//         }
//     }
// }
