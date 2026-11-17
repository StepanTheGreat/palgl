pub type ObjectId = u32;

pub struct Buffer {
    buffer: wgpu::Buffer,
    id: ObjectId
}

pub struct Texture {
    texture: wgpu::Texture,
    id: ObjectId
}

pub struct Sampler {
    sampler: wgpu::Sampler,
    id: ObjectId
}

pub struct Program {
    shader: wgpu::ShaderModule,
    id: ObjectId
}

pub struct RenderPipeline {
    pipeline: wgpu::RenderPipeline,
    id: ObjectId
}