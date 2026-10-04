use crate::settings::Settings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Renderer {
    OpenGl,
    Software,
}

pub fn core_variables(settings: &Settings, renderer: Renderer) -> Vec<(String, String)> {
    let scale = settings.video.scale.to_string();
    let render_mode = match renderer {
        Renderer::OpenGl => "opengl",
        Renderer::Software => "software",
    };
    let widescreen = if settings.video.widescreen && renderer == Renderer::OpenGl {
        "enabled"
    } else {
        "disabled"
    };
    let pinned = [
        ("melonds_render_mode", render_mode),
        ("melonds_widescreen", widescreen),
        ("melonds_opengl_resolution", scale.as_str()),
        ("melonds_boot_mode", "direct"),
        ("melonds_console_mode", "ds"),
        ("melonds_number_of_screen_layouts", "1"),
        ("melonds_screen_layout1", "top-bottom"),
        ("melonds_screen_gap", "0"),
        ("melonds_touch_mode", "auto"),
        ("melonds_show_cursor", "touching"),
        ("melonds_show_current_layout", "disabled"),
    ];
    pinned
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value_of<'a>(variables: &'a [(String, String)], key: &str) -> Option<&'a str> {
        variables
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    }

    #[test]
    fn headless_runs_use_the_software_renderer() {
        let variables = core_variables(&Settings::default(), Renderer::Software);
        assert_eq!(
            value_of(&variables, "melonds_render_mode"),
            Some("software")
        );
    }

    #[test]
    fn widescreen_needs_the_opengl_renderer() {
        let settings = Settings::default();
        assert_eq!(
            value_of(
                &core_variables(&settings, Renderer::OpenGl),
                "melonds_widescreen"
            ),
            Some("enabled")
        );
        assert_eq!(
            value_of(
                &core_variables(&settings, Renderer::Software),
                "melonds_widescreen"
            ),
            Some("disabled")
        );
    }

    #[test]
    fn scale_setting_drives_opengl_resolution() {
        let mut settings = Settings::default();
        settings.video.scale = 3;
        assert_eq!(
            value_of(
                &core_variables(&settings, Renderer::OpenGl),
                "melonds_opengl_resolution"
            ),
            Some("3")
        );
    }

    #[test]
    fn layout_is_pinned_to_gapless_top_bottom_because_touch_mapping_assumes_it() {
        let variables = core_variables(&Settings::default(), Renderer::OpenGl);
        assert_eq!(
            value_of(&variables, "melonds_screen_layout1"),
            Some("top-bottom")
        );
        assert_eq!(value_of(&variables, "melonds_screen_gap"), Some("0"));
    }
}
