//! Everything related to the global context - initialisation and everything

use std::{num::{NonZeroU32, NonZeroU64}, sync::Mutex};

use wgpu::util::DeviceExt;

use crate::rwh::{DisplayHandle, WindowHandle};

pub struct InitContext {
    pub raw_display_handle: Option<DisplayHandle>,
    pub raw_window_handle: WindowHandle
}

struct VertexBufferLayoutBuilder {
    array_stride: u64,
    step_mode: wgpu::VertexStepMode,
    attributes_start: usize,
    attributes_len: usize
}

impl VertexBufferLayoutBuilder {
    fn new(attributes_start: usize, array_stride: u64, step_mode: wgpu::VertexStepMode) -> Self {
        Self {
            array_stride,
            step_mode,
            attributes_start,
            attributes_len: 0
        }
    }
}

/// A vertex buffer layout builder. It's API is specifically made to simplify construction from C ABI.
/// 
/// You can reuse the same builder multiple times
/// 
/// ## Usage:
/// ```
/// let mut vblb = VertexBufferLayouts::new();
/// 
/// // Imagine that we would like to encode this layout:
/// // [position: float16x2][color: uint8x4]
/// 
/// // Push a new vertex buffer with a total stride of 8
/// vblb.push_slot_vertex_buffer(8, wgpu::VertexStepMode::Vertex);
/// 
/// // Add our positional vector of 2 16bit floats (right at the start)  
/// vblv.push_vertex_attribute(wgpu::VertexFormat::Float16x2, 0, 0);
/// 
/// // Add our color vector of 4 normalized uints (after the position)
/// vblv.push_vertex_attribute(wgpu::VertexFormat::Uint8x4, 4, 1);
/// 
/// // Vertex attributes are pushed onto the last buffer added to the builder
/// 
/// // Currently we have only one buffer
/// assert_eq(vblb.buffers(), 1);
/// 
/// // With two attributes
/// assert_eq(vblb.attributes_of(0), 2); 
/// 
/// ```
pub struct VertexBufferLayouts {
    buffers: Vec<Option<VertexBufferLayoutBuilder>>,
    attributes: Vec<wgpu::VertexAttribute>
}

impl VertexBufferLayouts {
    pub fn new() -> Self {
        Self {
            buffers: Vec::with_capacity(4),
            attributes: Vec::new()
        }
    }

    /// Push a vertex buffer layout slot
    pub fn push_slot_vertex_buffer(&mut self, array_stride: u64, step_mode: wgpu::VertexStepMode) {
        self.buffers.push(Some(VertexBufferLayoutBuilder::new(self.attributes.len(), array_stride, step_mode)));
    }

    /// Push an empty buffer slot. Some slots can be empty to allow shader reuse. 
    pub fn push_slot_void(&mut self) {
        self.buffers.push(None);
    }

    /// How many buffer slots are taken
    pub fn buffers(&self) -> usize {
        self.buffers.len()
    }

    /// Get the amount of attributes of a buffer. If the index is invalid or the buffer is sparse - returns 0.
    pub fn attributes_of(&self, buffer_slot: usize) -> usize {
        match self.buffers.get(buffer_slot) {
            Some(Some(buffer)) => buffer.attributes_len,
            _ => 0
        }
    }

    /// Push a vertex attribute onto the last buffer. 
    /// 
    /// Doesn't do anything if there are no buffers or the last buffer is void.
    pub fn push_vertex_attribute(&mut self, format: wgpu::VertexFormat, offset: u64, shader_location: u32) {
        if let Some(Some(buffer)) = self.buffers.last_mut() {
            self.attributes.push(wgpu::VertexAttribute {
                format,
                offset,
                shader_location
            });

            buffer.attributes_len += 1;
        }
    }

    pub fn clear(&mut self) {
        self.buffers.clear();
        self.attributes.clear();
    }

    fn build<'a>(&'a self) -> Vec<Option<wgpu::VertexBufferLayout<'a>>> {
        self.buffers.iter().map(|buffer| {
            match buffer {
                Some(buffer) => {
                    let attributes = &self.attributes[buffer.attributes_start..buffer.attributes_start+buffer.attributes_len];
                    Some(wgpu::VertexBufferLayout {
                        array_stride: buffer.array_stride,
                        step_mode: buffer.step_mode,
                        attributes
                    })
                },
                None => None
            }
        }).collect()
    }
}

#[repr(u32)]
pub enum BufferBindingType {
    Uniform = 1,
    Storage = 2,
    StorageReadOnly = 3,
}

impl From<BufferBindingType> for wgpu::BufferBindingType {
    fn from(value: BufferBindingType) -> Self {
        match value {
            BufferBindingType::Uniform => Self::Uniform,
            BufferBindingType::Storage => Self::Storage { read_only: false },
            BufferBindingType::StorageReadOnly => Self::Storage { read_only: true },
        }
    }
}

#[repr(u32)]
pub enum TextureSampleType {
    FloatFilterable = 1,
    FloatNonFilterable = 2,
    Depth = 3,
    Sint = 4,
    Uint = 5
}

impl From<TextureSampleType> for wgpu::TextureSampleType {
    fn from(value: TextureSampleType) -> Self {
        match value {
            TextureSampleType::Sint => Self::Sint,
            TextureSampleType::Uint => Self::Uint,
            TextureSampleType::Depth => Self::Depth,
            TextureSampleType::FloatFilterable => Self::Float { filterable: true },
            TextureSampleType::FloatNonFilterable => Self::Float { filterable: false },
        }
    }
}

pub struct BindGroupLayoutEntries {
    layouts: Vec<wgpu::BindGroupLayoutEntry>,
    binding_ty: Option<wgpu::BindingType>
}

impl BindGroupLayoutEntries {
    pub fn new() -> Self {
        Self { 
            layouts: Vec::new(),
            binding_ty: None
        }
    }

    /// Push a new layout with the currently selected binding type. Resets the internal binding type. 
    /// 
    /// Doesn't do anything if the binding type wasn't set. 
    pub fn push_layout(&mut self, binding: u32, visibility: wgpu::ShaderStages, count: u32) {
        if let Some(ty) = self.binding_ty.take() {
            self.layouts.push(wgpu::BindGroupLayoutEntry {
                binding,
                visibility,
                count: NonZeroU32::new(count),
                ty
            });
        }
    }

    pub fn set_binding_type_texture(&mut self, sample_type: TextureSampleType, view_dimension: wgpu::TextureViewDimension, multisampled: bool) {
        self.binding_ty = Some(wgpu::BindingType::Texture { 
            sample_type: sample_type.into(), 
            view_dimension,
            multisampled
        });
    }

    pub fn set_binding_type_buffer(&mut self, buffer_binding_type: BufferBindingType, has_dynamic_offset: bool, min_bindning_size: u64) {
        self.binding_ty = Some(wgpu::BindingType::Buffer { 
            ty: buffer_binding_type.into(), 
            has_dynamic_offset, 
            min_binding_size: NonZeroU64::new(min_bindning_size)
        });
    }

    pub fn set_binding_type_storage_texture(
        &mut self, 
        access: wgpu::StorageTextureAccess, 
        format: wgpu::TextureFormat, 
        view_dimension: wgpu::TextureViewDimension
    ) {
        self.binding_ty = Some(wgpu::BindingType::StorageTexture { 
            access, 
            format, 
            view_dimension
        })
    }

    pub fn set_binding_type_external_texture(&mut self) {
        self.binding_ty = Some(wgpu::BindingType::ExternalTexture);
    }

    pub fn set_binding_type_sampler(&mut self, ty: wgpu::SamplerBindingType) {
        self.binding_ty = Some(wgpu::BindingType::Sampler(ty));
    }

    pub fn layouts(&self) -> usize {
        self.layouts.len()
    }

    pub fn clear(&mut self) {
        self.layouts.clear();
    }

    fn build(&self) -> &[wgpu::BindGroupLayoutEntry] {
        &self.layouts
    }
}

#[repr(u32)]
pub enum IndexFormat {
    Uint16 = 1,
    Uint32 = 2
}

impl From<IndexFormat> for wgpu::IndexFormat {
    fn from(value: IndexFormat) -> Self {
        match value {
            IndexFormat::Uint16 => wgpu::IndexFormat::Uint16,
            IndexFormat::Uint32 => wgpu::IndexFormat::Uint32,
        }
    }
}

#[repr(u32)]
pub enum CompareFunction {
    Never = 1,
    Less = 2,
    Equal = 3,
    LessEqual = 4,
    Greater = 5,
    NotEqual = 6,
    GreaterEqual = 7,
    Always = 8,
}

impl From<CompareFunction> for wgpu::CompareFunction {
    fn from(value: CompareFunction) -> Self {
        match value {
            CompareFunction::Never => wgpu::CompareFunction::Never,
            CompareFunction::Less => wgpu::CompareFunction::Less,
            CompareFunction::Equal => wgpu::CompareFunction::Equal,
            CompareFunction::LessEqual => wgpu::CompareFunction::LessEqual,
            CompareFunction::Greater => wgpu::CompareFunction::Greater,
            CompareFunction::NotEqual => wgpu::CompareFunction::NotEqual,
            CompareFunction::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
            CompareFunction::Always => wgpu::CompareFunction::Always,
        }
    }
}

#[repr(u32)]
pub enum DepthWriteEnabled {
    True = 1,
    False = 2
}

impl From<DepthWriteEnabled> for bool {
    fn from(value: DepthWriteEnabled) -> Self {
        match value {
            DepthWriteEnabled::True => true,
            DepthWriteEnabled::False => false,
        }
    }
}

#[repr(u32)]
pub enum BlendState {
    Additive = 1,
    AlphaBlending = 2,
    PremultipliedAlphaBlending = 3,
    Replace = 4
}

impl From<BlendState> for wgpu::BlendState {
    fn from(value: BlendState) -> Self {
        match value {
            BlendState::Additive => wgpu::BlendState::ADDITIVE,
            BlendState::AlphaBlending => wgpu::BlendState::ALPHA_BLENDING,
            BlendState::Replace => wgpu::BlendState::REPLACE,
            BlendState::PremultipliedAlphaBlending => wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
        }
    }
}

#[derive(Default, Debug)]
pub struct RenderPipelineConfig {
    pub color_targets: Vec<Option<wgpu::ColorTargetState>>,
    pub primitive: wgpu::PrimitiveState,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    pub multisample: wgpu::MultisampleState
}

impl RenderPipelineConfig {
    pub fn new() -> Self {
        Self {
            color_targets: Vec::new(),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default()
        }
    }

    pub fn color_targets(&self) -> usize {
        self.color_targets.len()
    }

    /// Push a color target to use in fragment state
    pub fn push_color_target(&mut self, format: wgpu::TextureFormat, blend: Option<BlendState>, write_mask: wgpu::ColorWrites) {
        self.color_targets.push(Some(wgpu::ColorTargetState {
            format,
            blend: blend.map(|b| b.into()),
            write_mask
        }));
    }

    /// Push a void color target to use in fragment state
    pub fn push_void_color_target(&mut self) {
        self.color_targets.push(None);
    }

    /// 
    pub fn set_depth_stencil_state(
        &mut self, 
        format: wgpu::TextureFormat,
        depth_write_enabled: Option<DepthWriteEnabled>, 
        depth_compare: Option<CompareFunction>
    ) {
        match self.depth_stencil.as_mut() {
            Some(depth_state) => {
                depth_state.format = format;
                depth_state.depth_write_enabled = depth_write_enabled.map(|b| b.into());
                depth_state.depth_compare = depth_compare.map(|b| b.into());
            },
            None => {
                self.depth_stencil = Some(wgpu::DepthStencilState {
                    format,
                    depth_write_enabled: depth_write_enabled.map(|b| b.into()),
                    depth_compare: depth_compare.map(|b| b.into()),
                    bias: wgpu::DepthBiasState::default(),
                    stencil: wgpu::StencilState::default()
                });
            }
        }
    }

    /// Specify depth state stencil masks
    pub fn set_depth_state_stencil_masks(&mut self, read_mask: u32, write_mask: u32) {
        if let Some(depth_state) = self.depth_stencil.as_mut() {
            depth_state.stencil.read_mask = read_mask;
            depth_state.stencil.write_mask = write_mask;
        }
    }

    /// Specify stencil face state for a particular face
    pub fn set_depth_state_stencil_face(
        &mut self, 
        front_face: bool, 
        compare: wgpu::CompareFunction, 
        fail_op: wgpu::StencilOperation,
        depth_fail_op: wgpu::StencilOperation,
        pass_op: wgpu::StencilOperation
    ) {
        if let Some(depth_state) = self.depth_stencil.as_mut() {
            
            let face_state = wgpu::StencilFaceState {
                compare,
                fail_op,
                depth_fail_op,
                pass_op
            };

            if front_face {
                depth_state.stencil.front = face_state;
            } else {
                depth_state.stencil.back = face_state;
            }
        }
    }

    /// Configure primitive state
    pub fn set_primitive_state(
        &mut self, 
        topology: wgpu::PrimitiveTopology,
        front_face: wgpu::FrontFace,
        polygon_mode: wgpu::PolygonMode,
        strip_index_format: Option<IndexFormat>
    ) {
        self.primitive.front_face = front_face;
        self.primitive.topology = topology;
        self.primitive.polygon_mode = polygon_mode;
        self.primitive.strip_index_format = strip_index_format.map(|b| b.into());
    }

    /// Set multisampling settings
    pub fn set_multisample(&mut self, count: u32, mask: u64, alpha_to_coverage_enabled: bool) {
        self.multisample = wgpu::MultisampleState {
            count,
            alpha_to_coverage_enabled,
            mask
        };
    }

    /// Reset the builder
    pub fn reset(&mut self) {
        self.color_targets.clear();
        self.primitive = wgpu::PrimitiveState::default();
        self.depth_stencil = None;
        self.multisample = wgpu::MultisampleState::default();
    }
}

#[derive(Default, Debug)]
pub struct PalCtxConfig {
    pub backends: wgpu::Backends,
    pub power_preference: wgpu::PowerPreference,
    pub force_fallback_adapter: bool,
    pub required_features: wgpu::Features,
    pub required_limits: wgpu::Limits
}

/// A graphics context with all the neccessary graphics state
pub struct PalCtx {
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,

    /// The current command encoder
    command_encoder: Mutex<Option<wgpu::CommandEncoder>>,

    /// The current render pass. While present - nothing can be done with the command encoder
    render_pass: Mutex<Option<wgpu::RenderPass<'static>>>
}

impl PalCtx {
    /// # Safety
    /// Unsafe display/window handles could be passed when initializing the context.
    async unsafe fn new(init: InitContext, conf: &PalCtxConfig) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor { 
            display: None,
            backends: conf.backends,
            flags: wgpu::InstanceFlags::default(), 
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default()
        });

        let surface = unsafe { instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle { 
            raw_display_handle: init.raw_display_handle.map(|h| *h.handle()), 
            raw_window_handle: *init.raw_window_handle.handle()
        })}.unwrap();

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: conf.power_preference,
            force_fallback_adapter: conf.force_fallback_adapter,
            compatible_surface: None,
            apply_limit_buckets: false
        }).await.unwrap();

    
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: conf.required_features,
            required_limits: conf.required_limits.clone(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off
        }).await.unwrap();

        let command_encoder = Mutex::new(None);
        let render_pass = Mutex::new(None);

        Self {
            surface,
            adapter,
            device,
            queue,

            command_encoder,
            render_pass,
        }
    }

    /// Set a device lost callback
    pub fn set_device_lost_callback<F>(&self, f: F)
    where F: Fn(wgpu::DeviceLostReason) + Send + 'static  {
        self.device.set_device_lost_callback(move |reason, _| (f)(reason));
    }

    /// Check whether a render pass is currently active. While one is active, writing commands to a command encoder is impossible. 
    pub fn render_pass_active(&self) -> bool {
        self.render_pass.lock()
            .unwrap()
            .is_some()
    }

    /// Put a render pass
    fn set_render_pass(&self, rpass: wgpu::RenderPass<'static>) {
        let mut rpass_guard = self.render_pass.lock().unwrap();
        let _ = rpass_guard.replace(rpass);
    }

    fn drop_render_pass(&self) {
        let _ = self.render_pass.lock().unwrap().take();
    }

    /// Get a command encoder to write commands to (will lazily create one if one doesn't exist)
    fn with_command_encoder<F>(&self, f: F)
    where F: FnOnce(&mut wgpu::CommandEncoder) {
        
        // Don't do anything, we currently have a render pass active
        if self.render_pass_active() {
            return;
        }

        let mut encoder_guard = self.command_encoder.lock().unwrap();
        let encoder = encoder_guard.get_or_insert(
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default())
        );

        f(encoder);
    }

    /// Submit any work that was written to our command encoder
    fn submit_command_encoder(&self) {
        self.drop_render_pass();
        
        let mut encoder_guard = self.command_encoder.lock().unwrap();
        if let Some(encoder) = encoder_guard.take() {
            self.queue.submit([encoder.finish()]);
        } 
    }

    /// Create a new buffer from the provided size, usage and optional mapping parameter
    pub fn new_buffer(&self, size: u64, usage: wgpu::BufferUsages, mapped_at_creation: bool) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage,
            mapped_at_creation
        })
    }

    /// Create a new buffer with the provided contents
    pub fn new_buffer_data(&self, contents: &[u8], usage: wgpu::BufferUsages) -> wgpu::Buffer {
        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents,
            usage
        })
    }

    /// Create a new shader module
    pub fn new_shader(&self, wgsl: &str) -> wgpu::ShaderModule {
        self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(wgsl))
        })
    }

    /// Create a new texture with the provided parameters
    /// 
    /// - `size` = the dimensions of a texture (depending on a dimension it could mean different things)
    /// - `dimension` = how to interpret the size (A single 2D texture, an array of 2D textures, a cube or a 3D texture)
    /// - `mip_level_count` = how many mip layers this texture has (leave at 1 if there's none)
    /// - `sample_count` = whether the texture is multisampled (leave at 1 if not)
    pub fn new_texture(
        &self, 
        size: wgpu::Extent3d, 
        dimension: wgpu::TextureDimension,
        mip_level_count: u32,
        sample_count: u32, 
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages
    ) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count,
            sample_count,
            dimension,
            format,
            usage,
            view_formats: &[format]
        })
    }

    /// A simplified version of [PalCtx::new_texture] for 2D textures
    pub fn new_texture2d(
        &self, 
        size: (u32, u32), 
        mip_level_count: u32,
        sample_count: u32, 
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages
    ) -> wgpu::Texture {
        self.new_texture(
            wgpu::Extent3d{ width: size.0, height: size.1, depth_or_array_layers: 1}, 
            wgpu::TextureDimension::D2, 
            mip_level_count, 
            sample_count, 
            format, 
            usage
        )
    }

    /// A simplified version of [PalCtx::new_texture] for 3D textures
    pub fn new_texture3d(
        &self, 
        size: (u32, u32, u32),
        mip_level_count: u32, 
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages
    ) -> wgpu::Texture {
        self.new_texture(
            wgpu::Extent3d{ width: size.0, height: size.1, depth_or_array_layers: size.2}, 
            wgpu::TextureDimension::D3, 
            mip_level_count, 
            1, 
            format, 
            usage
        )
    }

    /// A simplified version of [PalCtx::new_texture] for 3D textures
    pub fn new_texture2d_array(
        &self, 
        size: (u32, u32),
        layers: u32, 
        mip_level_count: u32, 
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages
    ) -> wgpu::Texture {
        self.new_texture(
            wgpu::Extent3d{ width: size.0, height: size.1, depth_or_array_layers: layers}, 
            wgpu::TextureDimension::D2, 
            mip_level_count, 
            1, 
            format, 
            usage
        )
    }

    /// Create a new bind group layout
    pub fn new_bind_group_layout(&self, layout: &BindGroupLayoutEntries) -> wgpu::BindGroupLayout {
        self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: layout.build()
        })
    }

    /// Create a new pipeline layout 
    pub fn new_pipeline_layout(&self, layouts: &[Option<&wgpu::BindGroupLayout>]) -> wgpu::PipelineLayout {
        self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: layouts,
            immediate_size: 0
        })
    }

    /// Create a ne render pipeline from the provided shader, layout, vertex buffer layouts and primitive state
    pub fn new_render_pipeline(
        &self, 
        shader: &wgpu::ShaderModule, 
        layout: &wgpu::PipelineLayout,
        vertex_buffer_layouts: &VertexBufferLayouts,
        pipeline_config: &RenderPipelineConfig
    ) -> wgpu::RenderPipeline {
        let fragment = if !pipeline_config.color_targets.is_empty() {
            Some(wgpu::FragmentState {
                module: shader,
                entry_point: None,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &pipeline_config.color_targets
            })
        } else {
            None
        };

        self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: None,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &vertex_buffer_layouts.build()
            },
            fragment,
            primitive: pipeline_config.primitive,
            depth_stencil: pipeline_config.depth_stencil.clone(),
            multisample: pipeline_config.multisample,

            cache: None,
            multiview_mask: None
        })  
    }

    /// Write to a provided buffer with the provided offset
    pub fn write_buffer(&self, buffer: &wgpu::Buffer, offset: u64, data: &[u8]) {
        self.queue.write_buffer(buffer, offset, data);
    }

    /// Write tot he provided texture.
    /// 
    /// - `data` specifies the binary image data
    /// - `mip_level` to which mip level to write (defaults to the first one)
    /// - `aspect` which aspect of a texture (usually keep it at [None], only ever useful for depth/stencil textures to specify to which part to write)
    /// - `origin` where to start writing the texture
    /// - `size` how much to write
    pub fn write_texture(
        &self, 
        texture: &wgpu::Texture, 
        data: &[u8], 
        mip_level: Option<u32>,
        aspect: Option<wgpu::TextureAspect>,
        origin: wgpu::Origin3d,
        size: wgpu::Extent3d,
    ) {
        let aspect = aspect.unwrap_or(wgpu::TextureAspect::All);
        let mip_level = mip_level.unwrap_or(0);

        // To some textures we cannot write, or, the aspect is simply wrong
        let block_bytes = if let Some(s) = texture.format().block_copy_size(Some(aspect)) { s } else { return };

        let (block_width, block_height) = texture.format().block_dimensions();

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level,
                origin,
                aspect
            }, 
            data, 
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(texture.width().div_ceil(block_width) * block_bytes),
                rows_per_image: Some(texture.height().div_ceil(block_height))
            }, 
            size
        );
    }

    /// Similar to [PalCtx::write_texture], but writes to the entire texture. 
    pub fn write_texture_full(
        &self, 
        texture: &wgpu::Texture, 
        data: &[u8], 
        mip_level: Option<u32>,
        aspect: Option<wgpu::TextureAspect>,
    ) {
        self.write_texture(texture, data, mip_level, aspect, wgpu::Origin3d::ZERO, texture.size());
    }

    /// Begin a new render pass
    pub fn begin_render_pass(&self) {
        // Make sure that we first drop our existing render pass if one existed
        self.drop_render_pass();

        // Then issue a new one
        self.with_command_encoder(|enc| {
            let rpass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[],
                depth_stencil_attachment: None,
                ..Default::default()
            }).forget_lifetime();

            self.set_render_pass(rpass);
        });
    }

    /// End and submit the current render pass (if one existed)
    pub fn end_render_pass(&self) {
        self.drop_render_pass();
    }

    pub fn submit_frame(&self) {
        self.submit_command_encoder();

        todo!();
    }
}