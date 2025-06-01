#![feature(decl_macro)]

use anyhow::Result;
use tufa::{
    bindings::{StorageBuffer, UniformBuffer, mutability::Mutable},
    export::{
        egui::{self, ComboBox, Context, DragValue, Key},
        encase::ShaderType,
        nalgebra::Vector2,
        wgpu::{Features, RenderPass, ShaderModuleDescriptor, ShaderSource, ShaderStages},
        winit::{dpi::LogicalSize, window::WindowAttributes},
    },
    gpu::Gpu,
    interactive::{GraphicsCtx, Interactive},
    pipeline::{compute::ComputePipeline, render::RenderPipeline},
};

macro include_shader($($path:literal),+) {
    ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(
            concat!($(include_str!(concat!("../shaders/", $path)),)*).into(),
        ),
    }
}

#[derive(ShaderType, Default)]
struct Uniform {
    window: Vector2<u32>,
    domain: Vector2<u32>,
    tick: u32,
    view: u32,

    scale_factor: f32,
    pan: Vector2<f32>,
    zoom: f32,
    gain: f32,
    dt: f32,
}

#[derive(Clone, Copy, ShaderType)]
struct Cell {
    pressure: f32,
    velocity_x: f32,
    velocity_y: f32,
}

struct App {
    uniform: UniformBuffer<Uniform>,
    state: StorageBuffer<Vec<Cell>, Mutable>,

    render: RenderPipeline,
    divergence: ComputePipeline,
    advance: ComputePipeline,

    ctx: Uniform,
    running: bool,
}

impl Interactive for App {
    fn render(&mut self, gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        let window = gcx.window.inner_size();
        self.ctx.window = Vector2::new(window.width, window.height);
        self.ctx.scale_factor = gcx.window.scale_factor() as f32;

        self.uniform.upload(&self.ctx);
        self.render.draw_quad(render_pass, 0..1);
    }

    fn ui(&mut self, gcx: GraphicsCtx, ctx: &Context) {
        egui::Window::new("Fluid Sim")
            .default_width(0.0)
            .resizable(false)
            .show(ctx, |ui| {
                let dragging_viewport = ctx.dragged_id().is_none() && !ctx.is_pointer_over_area();
                let scale_factor = gcx.window.scale_factor() as f32;
                ctx.input(|input| {
                    if input.pointer.any_down() && dragging_viewport {
                        let delta = input.pointer.delta() * scale_factor;
                        self.ctx.pan += Vector2::new(delta.x, -delta.y);
                    }

                    self.running ^= input.key_pressed(Key::Space);
                    self.ctx.zoom += input.smooth_scroll_delta.y / 500.0;
                    self.ctx.view = (self.ctx.view + input.key_pressed(Key::Backslash) as u32) % 3;
                });

                ui.horizontal(|ui| {
                    ui.add(
                        DragValue::new(&mut self.ctx.gain)
                            .speed(0.01)
                            .range(0.0..=f32::MAX),
                    );
                    ui.label("Gain");
                });

                ui.horizontal(|ui| {
                    ui.add(
                        DragValue::new(&mut self.ctx.dt)
                            .speed(0.01)
                            .range(0.0..=f32::MAX),
                    );
                    ui.label("Delta Time");
                });

                ComboBox::from_label("View")
                    .selected_text(match self.ctx.view {
                        0 => "Pressure",
                        1 => "Velocity",
                        2 => "Divergence",
                        _ => unreachable!(),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.ctx.view, 0, "Pressure");
                        ui.selectable_value(&mut self.ctx.view, 1, "Velocity");
                        ui.selectable_value(&mut self.ctx.view, 2, "Divergence");
                    });

                ui.checkbox(&mut self.running, "Running");
                if ui.button("Step").clicked() || self.running {
                    let workgroups = self.ctx.domain.map(|x| x.div_ceil(8)).push(1);

                    for _ in 0..10 {
                        self.uniform.upload(&self.ctx);
                        self.divergence.dispatch(workgroups);
                        self.ctx.tick += 1;
                    }

                    self.uniform.upload(&self.ctx);
                    self.advance.dispatch(workgroups);
                    self.ctx.tick += 1;
                }
            });
    }
}

fn main() -> Result<()> {
    let gpu = Gpu::builder()
        .with_features(Features::SHADER_FLOAT32_ATOMIC)
        .build()?;
    let domain = Vector2::repeat(256);

    let mut state = vec![
        Cell {
            pressure: 0.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
        };
        (3 * domain.x * domain.y) as usize
    ];

    for (center, vel_x) in [
        (Vector2::new(128, 64_u32), 1.0),
        (Vector2::new(128, 192), -1.0),
    ] {
        for y in 0..domain.y {
            for x in 0..domain.x {
                let dist_sq = (y - center.x).pow(2) + (x - center.y).pow(2);
                if dist_sq < 16_u32.pow(2) {
                    state[(y * domain.x + x) as usize].velocity_x = vel_x;
                    state[(y * domain.x + x) as usize].pressure = 1.0;
                }
            }
        }
    }

    let uniform = gpu.create_uniform(&Uniform::default());
    let state = gpu.create_storage::<Vec<Cell>, Mutable>(&state);
    let render = gpu
        .render_pipeline(include_shader!("common.wgsl", "render.wgsl"))
        .bind(&uniform, ShaderStages::VERTEX_FRAGMENT)
        .bind(&state, ShaderStages::FRAGMENT)
        .finish();
    let divergence = gpu
        .compute_pipeline(include_shader!("common.wgsl", "divergence.wgsl"))
        .bind(&uniform)
        .bind(&state)
        .finish();
    let advance = gpu
        .compute_pipeline(include_shader!("common.wgsl", "advance.wgsl"))
        .bind(&uniform)
        .bind(&state)
        .finish();

    gpu.create_window(
        WindowAttributes::default()
            .with_title("Fluid Sim")
            .with_inner_size(LogicalSize::new(1920, 1080)),
        App {
            render,
            divergence,
            advance,

            uniform,
            state,

            running: false,
            ctx: Uniform {
                domain,
                zoom: 1.0,
                tick: 0,
                gain: 1.0,
                dt: 1.0,
                ..Uniform::default()
            },
        },
    )
    .run()?;

    Ok(())
}
