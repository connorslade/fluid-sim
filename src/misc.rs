use std::time::{SystemTime, UNIX_EPOCH};

pub macro include_shader($($path:literal),+) {
    tufa::export::wgpu::ShaderModuleDescriptor {
        label: None,
        source: tufa::export::wgpu::ShaderSource::Wgsl(
            concat!($(include_str!(concat!("../shaders/", $path)),)*).into(),
        ),
    }
}

pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
