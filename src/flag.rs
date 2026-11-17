macro_rules! decl_bit_constants {
    ($interface:ty => { $($(#[$meta:meta])* $name:ident => $value:ident),* $(,)? }) => {
        $(
            $(#[$meta])*
            pub const $name: $interface = <$interface>::$value;
        )*
    };
}

decl_bit_constants!(wgpu::BufferUsages => {
    /// Allow a buffer to be mapped for reading using [`Buffer::map_async`] + [`Buffer::get_mapped_range`].
    /// This does not include creating a buffer with [`BufferDescriptor::mapped_at_creation`] set.
    ///
    /// If [`Features::MAPPABLE_PRIMARY_BUFFERS`] isn't enabled, the only other usage a buffer
    /// may have is COPY_DST.
    BUFFER_USAGE_MAP_READ => MAP_READ,

    /// Allow a buffer to be mapped for writing using [`Buffer::map_async`] + [`Buffer::get_mapped_range_mut`].
    /// This does not include creating a buffer with [`BufferDescriptor::mapped_at_creation`] set.
    ///
    /// If [`Features::MAPPABLE_PRIMARY_BUFFERS`] feature isn't enabled, the only other usage a buffer
    /// may have is COPY_SRC.
    BUFFER_USAGE_MAP_WRITE => MAP_WRITE,

    /// Allow a buffer to be the source buffer for a [`CommandEncoder::copy_buffer_to_buffer`] or [`CommandEncoder::copy_buffer_to_texture`]
    /// operation.
    BUFFER_USAGE_COPY_SRC => COPY_SRC,

    /// Allow a buffer to be the destination buffer for a [`CommandEncoder::copy_buffer_to_buffer`], [`CommandEncoder::copy_texture_to_buffer`],
    /// [`CommandEncoder::clear_buffer`] or [`Queue::write_buffer`] operation.
    BUFFER_USAGE_COPY_DST => COPY_DST,

    /// Allow a buffer to be the index buffer in a draw operation.
    BUFFER_USAGE_INDEX => INDEX,

    /// Allow a buffer to be the vertex buffer in a draw operation.
    BUFFER_USAGE_VERTEX => VERTEX,

    /// Allow a buffer to be a [`BufferBindingType::Uniform`] inside a bind group.
    BUFFER_USAGE_UNIFORM => UNIFORM,

    /// Allow a buffer to be a [`BufferBindingType::Storage`] inside a bind group.
    BUFFER_USAGE_STORAGE => STORAGE,

    /// Allow a buffer to be the indirect buffer in an indirect draw call.
    BUFFER_USAGE_INDIRECT => INDIRECT,

    /// Allow a buffer to be the destination buffer for a [`CommandEncoder::resolve_query_set`] operation.
    BUFFER_USAGE_QUERY_RESOLVE => QUERY_RESOLVE,
});

decl_bit_constants!(wgpu::ShaderStages => {
    /// Binding is not visible from any shader stage.
    SHADER_STAGES_NONE => NONE,

    /// Binding is visible from the vertex shader of a render pipeline.
    SHADER_STAGES_VERTEX => VERTEX,

    /// Binding is visible from the fragment shader of a render pipeline.
    SHADER_STAGES_FRAGMENT => FRAGMENT,

    /// Binding is visible from the compute shader of a compute pipeline.
    SHADER_STAGES_COMPUTE => COMPUTE,

    /// Binding is visible from the vertex and fragment shaders of a render pipeline.
    SHADER_STAGES_VERTEX_FRAGMENT => VERTEX_FRAGMENT,
});

decl_bit_constants!(wgpu::TextureUsages => {
    /// Allows a texture to be the source in a [`CommandEncoder::copy_texture_to_buffer`] or
    /// [`CommandEncoder::copy_texture_to_texture`] operation.
    TEXTURE_STORAGE_COPY_SRC => COPY_SRC,

    /// Allows a texture to be the destination in a  [`CommandEncoder::copy_buffer_to_texture`],
    /// [`CommandEncoder::copy_texture_to_texture`], or [`Queue::write_texture`] operation.
    TEXTURE_STORAGE_COPY_DST => COPY_DST,

    /// Allows a texture to be a [`BindingType::Texture`] in a bind group.
    TEXTURE_STORAGE_TEXTURE_BINDING => TEXTURE_BINDING,

    /// Allows a texture to be a [`BindingType::StorageTexture`] in a bind group.
    TEXTURE_STORAGE_STORAGE_BINDING => STORAGE_BINDING,

    /// Allows a texture to be an output attachment of a render pass.
    ///
    /// Consider adding [`TextureUsages::TRANSIENT_ATTACHMENT`] if the contents are not reused.
    TEXTURE_STORAGE_RENDER_ATTACHMENT => RENDER_ATTACHMENT,
});

decl_bit_constants!(wgpu::Backends => {
    /// All the apis that wgpu offers first tier of support for.
    ///
    /// * [`Backends::VULKAN`]
    /// * [`Backends::METAL`]
    /// * [`Backends::DX12`]
    /// * [`Backends::BROWSER_WEBGPU`]
    BACKEND_PRIMARY => PRIMARY,

    /// All the apis that wgpu offers second tier of support for. These may
    /// be unsupported/still experimental.
    BACKEND_SECONDARY => SECONDARY,

    /// Supported on Windows, Linux/Android, and macOS/iOS via Vulkan Portability (with the Vulkan feature enabled)
    BACKEND_VULKAN => VULKAN,

    /// Supported on macOS and iOS.
    BACKEND_METAL => METAL,

    /// Supported on Windows 10 and later
    BACKEND_DX12 => DX12,

    /// Supported on Linux/Android, the web through webassembly via WebGL, and Windows and
    /// macOS/iOS via ANGLE
    BACKEND_GL => GL,
});