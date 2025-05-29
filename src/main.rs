use anyhow::Result;

use tufa::{
    bindings::{StorageBuffer, UniformBuffer, mutability::Mutable},
    export::{
        egui::{self, Context},
        encase::ShaderType,
        nalgebra::Vector2,
        wgpu::{RenderPass, ShaderStages, include_wgsl},
        winit::{dpi::LogicalSize, window::WindowAttributes},
    },
    gpu::Gpu,
    interactive::{GraphicsCtx, Interactive},
    pipeline::{compute::ComputePipeline, render::RenderPipeline},
};

#[derive(ShaderType, Default)]
struct Uniform {
    window: Vector2<u32>,
    domain: Vector2<u32>,
}

struct App {
    uniform: UniformBuffer<Uniform>,
    state: StorageBuffer<Vec<f32>, Mutable>,

    render: RenderPipeline,
    compute: ComputePipeline,

    ctx: Uniform,
}

impl Interactive for App {
    fn render(&mut self, gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        let window = gcx.window.inner_size();
        self.ctx.window = Vector2::new(window.width, window.height);

        self.uniform.upload(&self.ctx);
        self.render.draw_quad(render_pass, 0..1);
    }

    fn ui(&mut self, _gcx: GraphicsCtx, ctx: &Context) {
        egui::Window::new("Fluid Sim")
            .default_width(0.0)
            .resizable(false)
            .show(ctx, |ui| {
                if ui.button("Step").clicked() {
                    self.compute.dispatch(self.ctx.domain.push(1));
                }
            });
    }
}

fn main() -> Result<()> {
    let gpu = Gpu::new()?;
    let domain = Vector2::repeat(1000);

    let uniform = gpu.create_uniform(&Uniform::default());
    let state = gpu.create_storage_empty::<Vec<f32>, Mutable>((4 * domain.x * domain.y) as u64);
    let render = gpu
        .render_pipeline(include_wgsl!("../shaders/render.wgsl"))
        .bind(&uniform, ShaderStages::FRAGMENT)
        .bind(&state, ShaderStages::FRAGMENT)
        .finish();
    let compute = gpu
        .compute_pipeline(include_wgsl!("../shaders/compute.wgsl"))
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

            ctx: Uniform {
                domain,
                ..Uniform::default()
            },
        },
    )
    .run()?;

    Ok(())
}
