use tufa::export::egui::{self, ComboBox, Context, DragValue};

use crate::{App, types::View};

pub fn ui(this: &mut App, ctx: &Context) {
    egui::Window::new("Fluid Sim")
        .default_width(0.0)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("Simulation");

            ui.horizontal(|ui| {
                ui.add(
                    DragValue::new(&mut this.state.dt)
                        .speed(0.01)
                        .range(0.0..=f32::MAX),
                );
                ui.label("Delta Time");
            });

            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut this.state.iterations).range(1..=u32::MAX));
                ui.label("Iterations");
            });

            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut this.state.divergence));
                ui.label("Divergence");
            });

            ui.add_enabled_ui(!this.state.perf.is_empty(), |ui| {
                ui.collapsing("Performance", |ui| {
                    egui::Grid::new("performance_table")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Tick");
                            ui.label(format!("{:.2}ms", this.state.perf.avg_total() * 1000.0));
                            ui.end_row();

                            ui.label("Divergence");
                            ui.label(format!(
                                "{:.2}ms",
                                this.state.perf.avg_divergence() * 1000.0
                            ));
                            ui.end_row();

                            ui.label("Advance");
                            ui.label(format!("{:.2}ms", this.state.perf.avg_advance() * 1000.0));
                            ui.end_row();
                        });
                });
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                this.state.running ^= ui
                    .button(this.state.running.then_some("⏹ Stop").unwrap_or("▶ Run"))
                    .clicked();

                ui.add_enabled_ui(!this.state.running, |ui| {
                    (ui.button("⮫ Step").clicked() || this.state.running).then(|| this.tick())
                });
                ui.button("⟲ Reset").clicked().then(|| this.reset());
            });
            ui.button("💾 Download").clicked().then(|| this.download());

            ui.add_space(8.0);
            ui.heading("Rendering");

            ComboBox::from_label("View")
                .selected_text(this.state.view.name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut this.state.view, View::Pressure, "Pressure");
                    ui.selectable_value(&mut this.state.view, View::Velocity, "Velocity");
                    ui.selectable_value(&mut this.state.view, View::Divergance, "Divergence");
                });

            ui.horizontal(|ui| {
                ui.add(
                    DragValue::new(&mut this.state.gain[this.state.view as usize])
                        .speed(0.01)
                        .range(0.0..=f32::MAX),
                );
                ui.label("Gain");
            });

            if this.state.view == View::Velocity {
                ui.horizontal(|ui| {
                    ui.add(DragValue::new(&mut this.state.contours));
                    ui.label("Contours");
                });
            }
        });
}
