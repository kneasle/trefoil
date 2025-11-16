mod model;
mod utils;
mod viewport;

use three_d::*;

use crate::model::SimulationOptions;

const COLOR_THEME: catppuccin_egui::Theme = catppuccin_egui::MOCHA;

fn main() {
    // Create window
    let window = Window::new(WindowSettings {
        title: "Trefoil".to_string(),
        max_size: Some((1000, 800)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    // Create model
    let mut model = model::Model::polygon(8);
    let sim_opts = SimulationOptions::default();

    // Create model view
    let mut view = viewport::Viewport::new(&context, window.viewport());

    // Main loop
    let mut gui = three_d::GUI::new(&context);
    window.render_loop(move |mut frame_input| {
        // Update simulation
        let time_step = frame_input.elapsed_time as f32 / 1000.0;
        model.simulate(&sim_opts, time_step);

        // Render GUI
        let left_panel_width = 0.0;
        let mut right_panel_width = 0.0;
        let mut redraw = gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |egui_context| {
                catppuccin_egui::set_theme(egui_context, COLOR_THEME);
                // Right panel
                let response = egui::SidePanel::right("right-panel")
                    .min_width(250.0)
                    .show(egui_context, |ui| ui.heading("Settings"));
                right_panel_width = response.response.rect.width();
            },
        );

        // Calculate remaining viewport
        let wl = (left_panel_width * frame_input.device_pixel_ratio) as i32;
        let wr = (right_panel_width * frame_input.device_pixel_ratio) as i32;
        let width = frame_input.viewport.width as i32 - wl - wr;
        let viewport = Viewport {
            x: wl,
            y: 0,
            width: width.max(1) as u32,
            height: frame_input.viewport.height,
        };

        // Update the 3D view
        redraw |= view.update(&mut frame_input, viewport);
        if redraw {
            let screen = frame_input.screen();
            screen.clear(utils::clear_state_for_egui_color(COLOR_THEME.base));
            view.render(&model, &screen);
            screen.write(|| gui.render()).unwrap();
        }

        FrameOutput {
            swap_buffers: redraw,
            wait_next_event: !redraw,
            ..Default::default()
        }
    });
}
