#![feature(decl_macro)]
#![allow(clippy::obfuscated_if_else)]

use std::{fs, time::Instant};

use anyhow::Result;
use image::{GenericImageView, imageops::FilterType};
use misc::{include_shader, timestamp};
use tufa::{
    bindings::{
        StorageBuffer, UniformBuffer,
        mutability::{Immutable, Mutable},
    },
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
        let cells = (self.state.domain.x * self.state.domain.y) as usize;
        let mut state = vec![Cell::default(); 3 * cells];
        let mut walls = vec![0u32; cells.div_ceil(32)]; // todo: split walls to separate function
        scene(&mut state, &mut walls, self.state.domain);
        self.domain.upload(&state);
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

    pub fn download(&self) {
        let State { tick, domain, .. } = self.state;

        self.domain.download_async(move |state| {
            let count = domain.x * domain.y;
            let start_idx = (tick % 3 * count) as usize;
            let cells = &state[start_idx..(start_idx + count as usize)];

            let mut out = Vec::<u8>::with_capacity(4 * 3 * cells.len() + 4 * 2);
            out.extend(domain.x.to_le_bytes());
            out.extend(domain.y.to_le_bytes());
            for cell in cells {
                out.extend(cell.pressure.to_le_bytes());
                out.extend(cell.velocity_x.to_le_bytes());
                out.extend(cell.velocity_y.to_le_bytes());
            }

            let timestamp = timestamp();
            let _ = fs::create_dir_all("states");
            fs::write(format!("states/{timestamp}.bin"), out).unwrap();
        });
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
        let scale_factor = gcx.window.scale_factor() as f32;
        let pointer_over_ui = ctx.is_pointer_over_area();
        ctx.input(|input| {
            let dragging_viewport = input.pointer.middle_down() && !pointer_over_ui;
            if input.pointer.any_down() && dragging_viewport {
                let delta = input.pointer.delta() * scale_factor;
                self.state.pan += Vector2::new(delta.x, -delta.y);
            }

            self.state.zoom += input.smooth_scroll_delta.y / 500.0;
            self.state.running ^= input.key_pressed(Key::Space);

            input.key_pressed(Key::T).then(|| self.tick());
            input.key_pressed(Key::R).then(|| self.reset());
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
    let cells = (size.x * size.y) as usize;

    // Array of cells, making up the simulation domain
    let mut state = vec![Cell::default(); 3 * cells];
    // Bitfield denoting of each cell is a wall or not.
    let mut walls = vec![0u32; cells.div_ceil(32)];

    // Populate the state and wall bit-field with a default state
    scene(&mut state, &mut walls, size);

    let compute_uniform = gpu.create_uniform(&ComputeUniform::default());
    let render_uniform = gpu.create_uniform(&RenderUniform::default());
    let domain = gpu.create_storage::<Vec<Cell>, Mutable>(&state);
    let walls = gpu.create_storage::<Vec<u32>, Immutable>(&walls);
    let render = gpu
        .render_pipeline(include_shader!("common.wgsl", "render.wgsl"))
        .bind(&render_uniform, ShaderStages::VERTEX_FRAGMENT)
        .bind(&domain, ShaderStages::FRAGMENT)
        .bind(&walls, ShaderStages::FRAGMENT)
        .finish();
    let divergence = gpu
        .compute_pipeline(include_shader!("common.wgsl", "divergence.wgsl"))
        .bind(&compute_uniform)
        .bind(&domain)
        .bind(&walls)
        .finish();
    let advance = gpu
        .compute_pipeline(include_shader!("common.wgsl", "advance.wgsl"))
        .bind(&compute_uniform)
        .bind(&domain)
        .bind(&walls)
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

fn scene(state: &mut [Cell], walls: &mut [u32], size: Vector2<u32>) {
    let mask =
        image::open("airfoil-mask.png")
            .unwrap()
            .resize(size.x, size.y, FilterType::Triangle);
    for y in 0..size.y {
        for x in 0..size.x {
            if mask.get_pixel(x, y).0[0] < 128 {
                // todo: use a struct to wrap the bitvec
                let idx = (y * size.x + x) as usize;
                walls[idx / 32] |= 1 << (idx % 32);
            }
        }
    }
}
