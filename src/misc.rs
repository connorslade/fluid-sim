pub macro include_shader($($path:literal),+) {
    tufa::export::wgpu::ShaderModuleDescriptor {
        label: None,
        source: tufa::export::wgpu::ShaderSource::Wgsl(
            concat!($(include_str!(concat!("../shaders/", $path)),)*).into(),
        ),
    }
}
