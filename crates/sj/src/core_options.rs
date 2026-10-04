use sj_game::settings::Settings;

pub fn core_variables(settings: &Settings) -> Vec<(String, String)> {
    let scale = settings.video.scale.to_string();
    let pinned = [
        ("melonds_render_mode", "opengl"),
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
    fn scale_setting_drives_opengl_resolution() {
        let mut settings = Settings::default();
        settings.video.scale = 3;
        assert_eq!(
            value_of(&core_variables(&settings), "melonds_opengl_resolution"),
            Some("3")
        );
    }

    #[test]
    fn layout_is_pinned_to_gapless_top_bottom_because_touch_mapping_assumes_it() {
        let variables = core_variables(&Settings::default());
        assert_eq!(
            value_of(&variables, "melonds_screen_layout1"),
            Some("top-bottom")
        );
        assert_eq!(value_of(&variables, "melonds_screen_gap"), Some("0"));
    }
}
