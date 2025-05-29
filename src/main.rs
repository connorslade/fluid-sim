#![feature(decl_macro)]

use anyhow::Result;
use tufa::{
    bindings::{StorageBuffer, UniformBuffer, mutability::Mutable},
    export::{
        egui::{self, Context, DragValue, Key},
        encase::ShaderType,
        nalgebra::Vector2,
        wgpu::{RenderPass, ShaderModuleDescriptor, ShaderSource, ShaderStages},
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

    scale_factor: f32,
    pan: Vector2<f32>,
    zoom: f32,
    gain: f32,
}

struct App {
    uniform: UniformBuffer<Uniform>,
    state: StorageBuffer<Vec<f32>, Mutable>,

    render: RenderPipeline,
    compute: ComputePipeline,

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
                });

                ui.horizontal(|ui| {
                    ui.add(DragValue::new(&mut self.ctx.gain));
                    ui.label("Gain");
                });

                ui.checkbox(&mut self.running, "Running");
                if ui.button("Step").clicked() || self.running {
                    self.uniform.upload(&self.ctx);
                    self.compute
                        .dispatch(self.ctx.domain.map(|x| x.div_ceil(8)).push(1));
                    self.ctx.tick += 1;
                }
            });
    }
}

fn main() -> Result<()> {
    let gpu = Gpu::new()?;
    let domain = Vector2::repeat(1000);

    let mut state = vec![0.5; (3 * domain.x * domain.y) as usize];

    let center = Vector2::repeat(500_u32);
    for y in 0..domain.y {
        for x in 0..domain.x {
            let dist_sq = (y - center.x).pow(2) + (x - center.y).pow(2);
            state[(domain.x * domain.y + y * domain.x + x) as usize] =
                (dist_sq as f32).sqrt() / 250.0 - 1.0;
        }
    }

    let uniform = gpu.create_uniform(&Uniform::default());
    let state = gpu.create_storage::<Vec<f32>, Mutable>(&state);
    let render = gpu
        .render_pipeline(include_shader!("common.wgsl", "render.wgsl"))
        .bind(&uniform, ShaderStages::VERTEX_FRAGMENT)
        .bind(&state, ShaderStages::FRAGMENT)
        .finish();
    let compute = gpu
        .compute_pipeline(include_shader!("common.wgsl", "compute.wgsl"))
        .bind(&uniform)
        .bind(&state)
        .finish();

    gpu.create_window(
        WindowAttributes::default()
            .with_title("Fluid Sim")
            .with_inner_size(LogicalSize::new(1920, 1080)),
        App {
            render,
            compute,

            uniform,
            state,

            running: false,
            ctx: Uniform {
                domain,
                zoom: 1.0,
                tick: 1,
                gain: 1.0,
                ..Uniform::default()
            },
        },
    )
    .run()?;

    Ok(())
}
