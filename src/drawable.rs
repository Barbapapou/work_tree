use crate::Renderer;

pub trait Drawable {
    fn draw(&self, renderer: &Renderer);
}