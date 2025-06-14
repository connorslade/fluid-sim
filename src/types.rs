use std::time::Duration;

use encase::ShaderType;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use tufa::export::nalgebra::Vector2;

#[derive(Clone, Copy, Default, ShaderType)]
pub struct Cell {
    pub pressure: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
}

pub struct State {
    pub window: Vector2<u32>,
    pub scale_factor: f32,
    pub pan: Vector2<f32>,
    pub zoom: f32,
    pub view: View,

    pub gain: [f32; 3],
    pub contours: u32,

    pub domain: Vector2<u32>,
    pub tick: u32,
    pub dt: f32,

    pub perf: Performance,
    pub divergence: u32,
    pub iterations: u32,
    pub running: bool,
}

#[derive(Default)]
pub struct Performance {
    divergence: ConstGenericRingBuffer<f32, 64>,
    advance: ConstGenericRingBuffer<f32, 64>,
    total: ConstGenericRingBuffer<f32, 64>,
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq)]
pub enum View {
    Pressure,
    Velocity,
    Divergance,
}

#[derive(ShaderType, Default)]
pub struct RenderUniform {
    window: Vector2<u32>,
    domain: Vector2<u32>,
    scale_factor: f32,
    tick: u32,

    pan: Vector2<f32>,
    zoom: f32,
    gain: f32,
    view: u32,
    contours: u32,
}

#[derive(ShaderType, Default)]
pub struct ComputeUniform {
    domain: Vector2<u32>,
    tick: u32,
    dt: f32,
}

impl State {
    pub fn new(domain: Vector2<u32>) -> Self {
        State {
            window: Vector2::zeros(),
            scale_factor: 1.0,
            pan: Vector2::zeros(),
            zoom: 1.0,

            view: View::Pressure,
            gain: [1.0; 3],
            contours: 1,

            domain,
            tick: 0,
            dt: 1.0,

            divergence: 10,
            iterations: 1,
            running: false,
            perf: Performance::default(),
        }
    }

    pub fn render_uniform(&self) -> RenderUniform {
        RenderUniform {
            window: self.window,
            domain: self.domain,
            scale_factor: self.scale_factor,
            tick: self.tick - 1,

            pan: self.pan,
            zoom: self.zoom,
            gain: self.gain[self.view as usize],
            view: self.view as u32,
            contours: self.contours,
        }
    }

    pub fn compute_uniform(&self) -> ComputeUniform {
        ComputeUniform {
            domain: self.domain,
            tick: self.tick,
            dt: self.dt,
        }
    }
}

impl Performance {
    pub fn is_empty(&self) -> bool {
        self.divergence.is_empty() && self.advance.is_empty() && self.total.is_empty()
    }

    pub fn measure_divergence(&mut self, time: Duration) {
        self.divergence.push(time.as_secs_f32());
    }

    pub fn measure_advance(&mut self, time: Duration) {
        self.advance.push(time.as_secs_f32());
    }

    pub fn measure_total(&mut self, time: Duration) {
        self.total.push(time.as_secs_f32());
    }

    pub fn avg_divergence(&self) -> f32 {
        self.divergence.iter().sum::<f32>() / self.divergence.len() as f32
    }

    pub fn avg_advance(&self) -> f32 {
        self.advance.iter().sum::<f32>() / self.advance.len() as f32
    }

    pub fn avg_total(&self) -> f32 {
        self.total.iter().sum::<f32>() / self.total.len() as f32
    }
}

impl View {
    pub fn next(&mut self) {
        *self = match self {
            Self::Pressure => Self::Velocity,
            _ => Self::Pressure,
        };
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Pressure => "Pressure",
            Self::Velocity => "Velocity",
            Self::Divergance => "Divergence",
        }
    }
}
