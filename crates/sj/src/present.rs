use glow::HasContext;
use sj_game::image::RgbImage;
use sj_game::rect::RectPx;
use sj_game::screen_director::Screen;
use sj_game::settings::ScalingFilter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowOrder {
    BottomUp,
    TopDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameInfo {
    pub width_px: u32,
    pub height_px: u32,
    pub texture_width_px: u32,
    pub texture_height_px: u32,
    pub row_order: RowOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LastFrame {
    width_px: u32,
    height_px: u32,
    row_order: RowOrder,
}

pub struct Presenter {
    pub framebuffer: glow::Framebuffer,
    color: glow::Texture,
    depth: glow::Renderbuffer,
    capacity_px: (u32, u32),
    last_frame: Option<LastFrame>,
}

impl Presenter {
    pub fn new(gl: &glow::Context, width_px: u32, height_px: u32) -> Result<Self, String> {
        // SAFETY: plain GL object creation on the current context.
        let (framebuffer, color, depth) = unsafe {
            (
                gl.create_framebuffer()?,
                gl.create_texture()?,
                gl.create_renderbuffer()?,
            )
        };
        let mut presenter = Self {
            framebuffer,
            color,
            depth,
            capacity_px: (0, 0),
            last_frame: None,
        };
        presenter.ensure_capacity(gl, width_px, height_px);
        Ok(presenter)
    }

    pub fn ensure_capacity(&mut self, gl: &glow::Context, width_px: u32, height_px: u32) {
        if width_px <= self.capacity_px.0 && height_px <= self.capacity_px.1 {
            return;
        }
        let (width, height) = (width_px as i32, height_px as i32);
        // SAFETY: (re)allocates storage of objects we own and attaches them to our framebuffer.
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.color));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                width,
                height,
                0,
                glow::BGRA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(self.depth));
            gl.renderbuffer_storage(glow::RENDERBUFFER, glow::DEPTH24_STENCIL8, width, height);
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.framebuffer));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(self.color),
                0,
            );
            gl.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::DEPTH_STENCIL_ATTACHMENT,
                glow::RENDERBUFFER,
                Some(self.depth),
            );
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
        self.capacity_px = (width_px, height_px);
    }

    pub fn upload_software_frame(
        &mut self,
        gl: &glow::Context,
        width_px: u32,
        height_px: u32,
        xrgb_pixels: &[u32],
    ) {
        self.ensure_capacity(gl, width_px, height_px);
        let bytes: Vec<u8> = xrgb_pixels
            .iter()
            .flat_map(|pixel| pixel.to_le_bytes())
            .collect();
        // SAFETY: `bytes` holds exactly width*height BGRA pixels for a sub-image inside our texture.
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.color));
            gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                0,
                0,
                width_px as i32,
                height_px as i32,
                glow::BGRA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&bytes)),
            );
        }
        self.last_frame = Some(LastFrame {
            width_px,
            height_px,
            row_order: RowOrder::TopDown,
        });
    }

    pub fn read_frame(&self, gl: &glow::Context) -> Option<RgbImage> {
        let frame = self.last_frame?;
        let (width, height) = (frame.width_px as usize, frame.height_px as usize);
        let mut rgba = vec![0u8; width * height * 4];
        // SAFETY: `rgba` is exactly large enough for the frame rectangle inside our framebuffer.
        unsafe {
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(self.framebuffer));
            gl.read_pixels(
                0,
                0,
                width as i32,
                height as i32,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(Some(&mut rgba)),
            );
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
        }
        let mut rows: Vec<&[u8]> = rgba.chunks_exact(width * 4).collect();
        if frame.row_order == RowOrder::BottomUp {
            rows.reverse();
        }
        let rgb = rows
            .iter()
            .flat_map(|row| {
                row.as_chunks::<4>()
                    .0
                    .iter()
                    .flat_map(|pixel| [pixel[0], pixel[1], pixel[2]])
            })
            .collect();
        Some(RgbImage {
            width_px: frame.width_px,
            height_px: frame.height_px,
            rgb,
        })
    }

    pub fn note_hardware_frame(&mut self, width_px: u32, height_px: u32) {
        self.last_frame = Some(LastFrame {
            width_px,
            height_px,
            row_order: RowOrder::BottomUp,
        });
    }

    pub fn color_texture(&self) -> glow::Texture {
        self.color
    }

    pub fn frame_info(&self) -> Option<FrameInfo> {
        let frame = self.last_frame?;
        Some(FrameInfo {
            width_px: frame.width_px,
            height_px: frame.height_px,
            texture_width_px: self.capacity_px.0,
            texture_height_px: self.capacity_px.1,
            row_order: frame.row_order,
        })
    }

    pub fn draw(
        &self,
        gl: &glow::Context,
        drawable_px: (u32, u32),
        screens: &[(Screen, RectPx, (f32, f32))],
        filter: ScalingFilter,
    ) {
        // SAFETY: clears the default framebuffer of the current context.
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.disable(glow::SCISSOR_TEST);
            gl.viewport(0, 0, drawable_px.0 as i32, drawable_px.1 as i32);
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
        let Some(frame) = self.last_frame else {
            return;
        };
        let screen_height_px = frame.height_px / 2;
        let top_screen = RectPx {
            x: 0,
            y: 0,
            width: frame.width_px,
            height: screen_height_px,
        };
        let bottom_screen = RectPx {
            y: screen_height_px as i32,
            ..top_screen
        };
        let gl_filter = gl_filter(filter);
        for (screen, destination, span) in screens {
            let whole = match screen {
                Screen::Top => top_screen,
                Screen::Bottom => bottom_screen,
            };
            let source = RectPx {
                x: (span.0 * frame.width_px as f32).round() as i32,
                width: ((span.1 - span.0) * frame.width_px as f32).round() as u32,
                ..whole
            };
            self.blit(gl, frame, source, *destination, drawable_px.1, gl_filter);
        }
    }

    fn blit(
        &self,
        gl: &glow::Context,
        frame: LastFrame,
        source: RectPx,
        destination: RectPx,
        drawable_height_px: u32,
        gl_filter: u32,
    ) {
        let (source_y0, source_y1) = source_rows(source, frame.height_px, frame.row_order);
        let destination_bottom =
            drawable_height_px as i32 - destination.y - destination.height as i32;
        // SAFETY: blits between our framebuffer and the default one, both colour-complete.
        unsafe {
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(self.framebuffer));
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);
            gl.blit_framebuffer(
                source.x,
                source_y0,
                source.x + source.width as i32,
                source_y1,
                destination.x,
                destination_bottom,
                destination.x + destination.width as i32,
                destination_bottom + destination.height as i32,
                glow::COLOR_BUFFER_BIT,
                gl_filter,
            );
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
        }
    }
}

pub fn read_default_framebuffer(gl: &glow::Context, drawable_px: (u32, u32)) -> Vec<u8> {
    let mut rgba = vec![0u8; drawable_px.0 as usize * drawable_px.1 as usize * 4];
    // SAFETY: `rgba` is exactly large enough for the requested RGBA rectangle.
    unsafe {
        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
        gl.read_pixels(
            0,
            0,
            drawable_px.0 as i32,
            drawable_px.1 as i32,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelPackData::Slice(Some(&mut rgba)),
        );
    }
    rgba
}

fn gl_filter(filter: ScalingFilter) -> u32 {
    match filter {
        ScalingFilter::Smooth => glow::LINEAR,
        ScalingFilter::Sharp => glow::NEAREST,
    }
}

// GL blits map the first source row to the bottom of the destination, so we always hand over
// the image's bottom row first. Core-rendered frames are stored bottom-up, uploaded ones top-down.
fn source_rows(region: RectPx, frame_height_px: u32, row_order: RowOrder) -> (i32, i32) {
    let region_top = region.y;
    let region_bottom = region.y + region.height as i32;
    match row_order {
        RowOrder::TopDown => (region_bottom, region_top),
        RowOrder::BottomUp => {
            let stored_bottom = frame_height_px as i32 - region_bottom;
            (stored_bottom, stored_bottom + region.height as i32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOP_HALF: RectPx = RectPx {
        x: 0,
        y: 0,
        width: 256,
        height: 192,
    };
    const BOTTOM_HALF: RectPx = RectPx {
        x: 0,
        y: 192,
        width: 256,
        height: 192,
    };

    #[test]
    fn uploaded_frames_are_flipped_by_reversing_rows() {
        assert_eq!(source_rows(TOP_HALF, 384, RowOrder::TopDown), (192, 0));
        assert_eq!(source_rows(BOTTOM_HALF, 384, RowOrder::TopDown), (384, 192));
    }

    #[test]
    fn core_rendered_frames_keep_row_order_but_swap_halves() {
        assert_eq!(source_rows(TOP_HALF, 384, RowOrder::BottomUp), (192, 384));
        assert_eq!(source_rows(BOTTOM_HALF, 384, RowOrder::BottomUp), (0, 192));
    }
}
