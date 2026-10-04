use sdl2::event::Event;

// egui-sdl2 treats SDL mouse positions as drawable pixels, but on high-DPI macOS SDL reports
// window points, so clicks landed at half their height. We convert before handing events over.
pub fn mouse_event_in_drawable_pixels(
    event: &Event,
    to_drawable: impl Fn(i32, i32) -> (i32, i32),
) -> Event {
    let mut converted = event.clone();
    match &mut converted {
        Event::MouseButtonDown { x, y, .. }
        | Event::MouseButtonUp { x, y, .. }
        | Event::MouseMotion { x, y, .. } => (*x, *y) = to_drawable(*x, *y),
        _ => {}
    }
    converted
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdl2::mouse::{MouseButton, MouseState};

    fn retina(x: i32, y: i32) -> (i32, i32) {
        (x * 2, y * 2)
    }

    #[test]
    fn mouse_clicks_and_motion_are_scaled_to_drawable_pixels() {
        let click = Event::MouseButtonDown {
            timestamp: 0,
            window_id: 1,
            which: 0,
            mouse_btn: MouseButton::Left,
            clicks: 1,
            x: 100,
            y: 255,
        };
        let Event::MouseButtonDown { x, y, .. } = mouse_event_in_drawable_pixels(&click, retina)
        else {
            panic!("event kind changed");
        };
        assert_eq!((x, y), (200, 510));

        let motion = Event::MouseMotion {
            timestamp: 0,
            window_id: 1,
            which: 0,
            mousestate: MouseState::from_sdl_state(0),
            x: 10,
            y: 20,
            xrel: 1,
            yrel: 1,
        };
        let Event::MouseMotion { x, y, xrel, .. } = mouse_event_in_drawable_pixels(&motion, retina)
        else {
            panic!("event kind changed");
        };
        assert_eq!((x, y, xrel), (20, 40, 1));
    }

    #[test]
    fn other_events_pass_through_untouched() {
        let quit = Event::Quit { timestamp: 7 };
        assert_eq!(mouse_event_in_drawable_pixels(&quit, retina), quit);
    }
}
