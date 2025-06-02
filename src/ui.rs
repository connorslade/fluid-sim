use tufa::export::egui::{self, ComboBox, Context, DragValue};

use crate::{App, scene, types::Cell};

pub fn ui(this: &mut App, ctx: &Context) {
    let state = &mut this.state;

    egui::Window::new("Fluid Sim")
        .default_width(0.0)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("Simulation");

            ui.horizontal(|ui| {
                ui.add(
                    DragValue::new(&mut state.dt)
                        .speed(0.01)
                        .range(0.0..=f32::MAX),
                );
                ui.label("Delta Time");
            });

            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut state.iterations));
                ui.label("Iterations");
            });

            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut state.divergence_iterations));
                ui.label("Divergence");
            });

            ui.horizontal(|ui| {
                if ui.button("Step").clicked() || state.running {
                    let workgroups = state.domain.map(|x| x.div_ceil(8)).push(1);

                    for _ in 0..state.iterations {
                        for _ in 0..state.divergence_iterations {
                            this.compute_uniform.upload(&state.compute_uniform());
                            this.divergence.dispatch(workgroups);
                            state.tick += 1;
                        }

                        this.compute_uniform.upload(&state.compute_uniform());
                        this.advance.dispatch(workgroups);
                        state.tick += 1;
                    }
                }

                if ui.button("Reset").clicked() {
                    state.running = false;
                    let mut cells =
                        vec![Cell::default(); (3 * state.domain.x * state.domain.y) as usize];
                    scene(&mut cells, state.domain);
                    this.domain.upload(&cells);
                    state.tick = 0;
                }
            });

            ui.add_space(8.0);
            ui.heading("Rendering");

            ComboBox::from_label("View")
                .selected_text(match state.view {
                    0 => "Pressure",
                    1 => "Velocity",
                    2 => "Divergence",
                    _ => unreachable!(),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.view, 0, "Pressure");
                    ui.selectable_value(&mut state.view, 1, "Velocity");
                    ui.selectable_value(&mut state.view, 2, "Divergence");
                });

            ui.horizontal(|ui| {
                ui.add(
                    DragValue::new(&mut state.gain)
                        .speed(0.01)
                        .range(0.0..=f32::MAX),
                );
                ui.label("Gain");
            });
        });
}
