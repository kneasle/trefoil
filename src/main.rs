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
    let mut model = model::Model::polygon(8, 2);
    model.add_polygon(8, &vec![1, 2]);
    model.add_polygon(6, &vec![9, 2, 3]);
    model.add_polygon(6, &vec![0, 1, 4]);

    let unsimulated_model = model.clone();

    // Gui variables
    let mut is_simulating = false;
    let mut sim_opts = SimulationOptions::default();

    // Create model view
    let mut view = viewport::Viewport::new(&context, window.viewport());

    // Main loop
    let mut gui = three_d::GUI::new(&context);
    window.render_loop(move |mut frame_input| {
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
                let response = egui::SidePanel::right("right-panel").min_width(250.0).show(
                    egui_context,
                    |ui| {
                        sidebar_gui(
                            ui,
                            &mut is_simulating,
                            &mut sim_opts,
                            &mut model,
                            &unsimulated_model,
                        )
                    },
                );
                right_panel_width = response.response.rect.width();
            },
        );

        // Update simulation
        if is_simulating {
            let time_step = frame_input.elapsed_time as f32 / 1000.0;
            model.simulate(&sim_opts, time_step);
        }

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
        redraw |= is_simulating;
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

fn sidebar_gui(
    ui: &mut egui::Ui,
    is_simulating: &mut bool,
    _sim_opts: &mut SimulationOptions,
    model: &mut crate::model::Model,
    prev_model: &crate::model::Model,
) {
    ui.heading(format!(
        "Simulation is {}",
        if *is_simulating { "running" } else { "paused" }
    ));

    let button_text = if *is_simulating { "Pause" } else { "Play" };
    if ui.button(button_text).clicked() {
        *is_simulating = !*is_simulating;
    }

    if ui.button("Reset").clicked() {
        *model = prev_model.clone();
    }
}
