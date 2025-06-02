#![feature(decl_macro)]
#![allow(clippy::obfuscated_if_else)]

use std::time::Instant;

use anyhow::Result;
use misc::include_shader;
use tufa::{
    bindings::{StorageBuffer, UniformBuffer, mutability::Mutable},
    export::{
        egui::{Context, Key},
        nalgebra::Vector2,
        wgpu::{Features, RenderPass, ShaderStages},
        winit::window::WindowAttributes,
    },
    gpu::Gpu,
    interactive::{GraphicsCtx, Interactive},
    pipeline::{compute::ComputePipeline, render::RenderPipeline},
};
use types::{Cell, ComputeUniform, RenderUniform, State};

mod misc;
mod types;
mod ui;

struct App {
    render_uniform: UniformBuffer<RenderUniform>,
    compute_uniform: UniformBuffer<ComputeUniform>,
    domain: StorageBuffer<Vec<Cell>, Mutable>,

    render: RenderPipeline,
    divergence: ComputePipeline,
    advance: ComputePipeline,

    state: State,
}

impl App {
    pub fn reset(&mut self) {
        self.state.running = false;
        let mut cells =
            vec![Cell::default(); (3 * self.state.domain.x * self.state.domain.y) as usize];
        scene(&mut cells, self.state.domain);
        self.domain.upload(&cells);
        self.state.tick = 0;
    }

    pub fn tick(&mut self) {
        let workgroups = self.state.domain.map(|x| x.div_ceil(8)).push(1);

        let time = Instant::now();
        for _ in 0..self.state.iterations {
            let time = Instant::now();
            for _ in 0..self.state.divergence {
                self.compute_uniform.upload(&self.state.compute_uniform());
                self.divergence.dispatch(workgroups);
                self.state.tick += 1;
            }
            self.state
                .perf
                .measure_divergence(time.elapsed() / self.state.divergence);

            let time = Instant::now();
            self.compute_uniform.upload(&self.state.compute_uniform());
            self.advance.dispatch(workgroups);
            self.state.tick += 1;
            self.state.perf.measure_advance(time.elapsed());
        }
        self.state
            .perf
            .measure_total(time.elapsed() / self.state.iterations);
    }
}

impl Interactive for App {
    fn render(&mut self, gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        let window = gcx.window.inner_size();
        self.state.window = Vector2::new(window.width, window.height);
        self.state.scale_factor = gcx.window.scale_factor() as f32;

        self.render_uniform.upload(&self.state.render_uniform());
        self.render.draw_quad(render_pass, 0..1);
    }

    fn ui(&mut self, gcx: GraphicsCtx, ctx: &Context) {
        let dragging_viewport = ctx.dragged_id().is_none() && !ctx.is_pointer_over_area();
        let scale_factor = gcx.window.scale_factor() as f32;
        ctx.input(|input| {
            if input.pointer.any_down() && dragging_viewport {
                let delta = input.pointer.delta() * scale_factor;
                self.state.pan += Vector2::new(delta.x, -delta.y);
            }

            self.state.running ^= input.key_pressed(Key::Space);
            self.state.zoom += input.smooth_scroll_delta.y / 500.0;
            input.key_pressed(Key::Backspace).then(|| self.reset());
            input
                .key_pressed(Key::Backslash)
                .then(|| self.state.view.next());
        });

        ui::ui(self, ctx);
    }
}

fn main() -> Result<()> {
    let gpu = Gpu::builder()
        .with_features(Features::SHADER_FLOAT32_ATOMIC)
        .build()?;

    let size = Vector2::repeat(256);
    let mut state = vec![Cell::default(); (3 * size.x * size.y) as usize];
    scene(&mut state, size);

    let compute_uniform = gpu.create_uniform(&ComputeUniform::default());
    let render_uniform = gpu.create_uniform(&RenderUniform::default());
    let domain = gpu.create_storage::<Vec<Cell>, Mutable>(&state);
    let render = gpu
        .render_pipeline(include_shader!("common.wgsl", "render.wgsl"))
        .bind(&render_uniform, ShaderStages::VERTEX_FRAGMENT)
        .bind(&domain, ShaderStages::FRAGMENT)
        .finish();
    let divergence = gpu
        .compute_pipeline(include_shader!("common.wgsl", "divergence.wgsl"))
        .bind(&compute_uniform)
        .bind(&domain)
        .finish();
    let advance = gpu
        .compute_pipeline(include_shader!("common.wgsl", "advance.wgsl"))
        .bind(&compute_uniform)
        .bind(&domain)
        .finish();

    gpu.create_window(
        WindowAttributes::default().with_title("Fluid Sim"),
        App {
            render,
            divergence,
            advance,

            render_uniform,
            compute_uniform,

            domain,
            state: State::new(size),
        },
    )
    .run()?;

    Ok(())
}

fn scene(state: &mut [Cell], size: Vector2<u32>) {
    for (center, vel_x) in [
        (Vector2::new(128, 64_u32), 1.0),
        (Vector2::new(128, 192), -1.0),
    ] {
        for y in 0..size.y {
            for x in 0..size.x {
                let dist_sq = (y - center.x).pow(2) + (x - center.y).pow(2);
                if dist_sq < 16_u32.pow(2) {
                    state[(y * size.x + x) as usize].velocity_x = vel_x;
                    state[(y * size.x + x) as usize].pressure = 1.0;
                }
            }
        }
    }
}
