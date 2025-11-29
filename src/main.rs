mod model;
mod utils;
mod viewport;

use three_d::*;

use crate::{model::SimulationOptions, viewport::RenderOptions};

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
    model.add_polygon(8, &vec![(0, 0), (1, 0)], Some(Vec3::new(1.0, 0.0, 1.0)));
    model.add_polygon(
        6,
        &vec![(2, 0), (1, 0), (9, 0)],
        Some(Vec3::new(0.0, 1.0, -1.0)),
    );
    model.add_polygon(
        6,
        &vec![(4, 0), (0, 0), (3, 1)],
        Some(Vec3::new(0.0, -1.0, -1.0)),
    );
    model.add_polygon(
        6,
        &vec![(15, 0), (3, 1), (2, 1), (10, 1)],
        Some(Vec3::new(0.0, -1.0, 0.0)),
    );
    model.add_polygon(5, &vec![(16, 0), (17, 0)], Some(Vec3::new(0.0, -1.0, 1.0)));
    model.add_polygon(6, &vec![(20, 0), (17, 0), (10, 1), (11, 1)], None);
    model.add_polygon(6, &vec![(18, 0), (19, 0), (19, 1), (20, 1), (21, 1)], None);
    model.connect_edge((19, 0), (19, 1));

    let unsimulated_model = model.clone();

    // Gui variables
    let mut is_simulating = false;
    let mut sim_opts = SimulationOptions::default();
    let mut render_opts = RenderOptions {
        show_axes: true,
        show_vert_indices: true,
    };

    // Create model view
    let mut view = crate::viewport::Viewport::new(&context, window.viewport());

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
                            &mut render_opts,
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
            view.render(&model, &render_opts, &screen);
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
    sim_opts: &mut SimulationOptions,
    render_opts: &mut RenderOptions,
    model: &mut crate::model::Model,
    prev_model: &crate::model::Model,
) {
    ui.heading(format!(
        "Simulation is {}",
        if *is_simulating { "running" } else { "paused" }
    ));
    let button_text = if *is_simulating { "Pause" } else { "Play" };
    if ui.button(button_text).clicked() {
        if !*is_simulating {
            // Add the extra vertices if we're starting a simulation
            model.ensure_all_verts_have_three_edges();
        }
        *is_simulating = !*is_simulating;
    }
    if ui.button("Reset").clicked() {
        *model = prev_model.clone();
        *is_simulating = false;
    }

    ui.add_space(20.0);
    ui.heading("Simulation options");
    ui.horizontal(|ui| {
        ui.label("Edge length force:");
        ui.add(
            egui::Slider::new(&mut sim_opts.edge_length_force, 0.01..=10.0)
                .show_value(true)
                .logarithmic(true),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Vertex angle force:");
        ui.add(
            egui::Slider::new(&mut sim_opts.vertex_angle_force, 0.01..=10.0)
                .show_value(true)
                .logarithmic(true),
        );
    });

    ui.add_space(20.0);
    ui.heading("Rendering options");
    ui.checkbox(&mut render_opts.show_axes, "Show axes");
    ui.checkbox(&mut render_opts.show_vert_indices, "Show vertex indices");
}
