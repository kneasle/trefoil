use three_d::{egui::Color32, *};

pub fn clear_state_for_egui_color(clear_color: Color32) -> three_d::ClearState {
    let clear_state = three_d::ClearState::color_and_depth(
        clear_color.r() as f32 / 255.0,
        clear_color.g() as f32 / 255.0,
        clear_color.b() as f32 / 255.0,
        1.0,
        1.0,
    );
    clear_state
}

pub fn egui_color_to_srgba(c: Color32) -> Srgba {
    let [r, g, b, a] = c.to_srgba_unmultiplied();
    Srgba { r, g, b, a }
}
