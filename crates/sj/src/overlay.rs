use sdl2::event::Event;

use crate::mouse_points::mouse_event_in_drawable_pixels;
use crate::platform::Platform;

pub struct Overlay {
    egui: egui_sdl2::EguiGlow,
}

impl Overlay {
    pub fn new(platform: &Platform) -> Self {
        Self {
            egui: egui_sdl2::EguiGlow::new(&platform.window, platform.gl.clone(), None, false),
        }
    }

    pub fn handle_event(&mut self, platform: &Platform, event: &Event) {
        let event = mouse_event_in_drawable_pixels(event, |x, y| platform.to_drawable(x, y));
        let _ = self.egui.on_event(&platform.window, &event);
    }

    pub fn wants_keyboard(&self) -> bool {
        self.egui.ctx.egui_wants_keyboard_input()
    }

    pub fn wants_pointer(&self) -> bool {
        self.egui.ctx.egui_wants_pointer_input() || self.egui.ctx.is_pointer_over_egui()
    }

    pub fn register_native_texture(&mut self, texture: glow::Texture) -> egui::TextureId {
        self.egui.painter.register_native_texture(texture)
    }

    pub fn clear(&self, rgba: [f32; 4]) {
        self.egui.clear(rgba);
    }

    pub fn draw(&mut self, show: impl FnMut(&mut egui::Ui)) {
        self.egui.run_ui(show);
        self.egui.paint();
    }
}

impl Drop for Overlay {
    fn drop(&mut self) {
        self.egui.destroy();
    }
}
