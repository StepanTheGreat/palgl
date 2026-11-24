//! Everything related to the global context - initialisation and everything

use wgpu::util::DeviceExt;

use crate::{builders::{BindGroupLayoutEntries, RenderPassBuilder, RenderPipelineConfig, VertexBufferLayouts}, object::{BufferId, ObjectStorage, RPipelineId, ShaderId, TextureId}, rwh::{DisplayHandle, WindowHandle}, values::{BlendState, CompareFunction, DepthWriteEnabled, IndexFormat}};

pub struct InitContext {
    pub raw_display_handle: Option<DisplayHandle>,
    pub raw_window_handle: WindowHandle
}

#[derive(Debug)]
pub struct PalCtxConfig {
    pub init_width: u32,
    pub init_height: u32,

    pub backends: wgpu::Backends,
    pub power_preference: wgpu::PowerPreference,
    pub force_fallback_adapter: bool,
    pub required_features: wgpu::Features,
    pub required_limits: wgpu::Limits
}

impl Default for PalCtxConfig {
    fn default() -> Self {
        Self {
            init_height: 512,
            init_width: 512,

            backends: wgpu::Backends::default(),
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            required_features: wgpu::Features::default(),
            required_limits: wgpu::Limits::default()
        }
    }
}

/// A graphics context with all the neccessary graphics state
pub struct PalCtx {
    surface: Option<wgpu::Surface<'static>>,
    device: wgpu::Device,
    queue: wgpu::Queue,

    objects: ObjectStorage,
    
    /// A permanent surface ID that's mapped to a texture ID. This allows the user to retrieve the current surface as a texture object. 
    surface_id: TextureId,
    surface_texture: Option<wgpu::SurfaceTexture>,
    surface_conf: wgpu::SurfaceConfiguration,
    new_surface_conf: Option<wgpu::SurfaceConfiguration>,

    /// The current command encoder
    command_encoder: Option<wgpu::CommandEncoder>,

    /// The current render pass. While present - nothing can be done with the command encoder
    render_pass: Option<wgpu::RenderPass<'static>>
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

        let command_encoder = None;
        let render_pass = None;

        
        let mut objects = ObjectStorage::new();
        
        let surface_conf = {
            let caps = surface.get_capabilities(&adapter);
            wgpu::SurfaceConfiguration {
                usage: caps.usages,
                format: caps.formats[0],
                alpha_mode: caps.alpha_modes[0],
                width: conf.init_width,
                height: conf.init_height,
                present_mode: wgpu::PresentMode::Fifo,
                view_formats: vec![],
                color_space: wgpu::SurfaceColorSpace::Auto,
                desired_maximum_frame_latency: 2
            }
        };

        let surface = Some(surface);
        let surface_id = objects.new_surface_texture();
        let surface_texture = None;

        let new_surface_conf = None;

        let mut this = Self {
            surface,
            device,
            queue,

            objects,
            surface_id,
            surface_conf,
            surface_texture,
            new_surface_conf,

            command_encoder,
            render_pass,
        };

        this.update_and_grab_surface();

        this
    }

    /// Set a device lost callback
    pub fn set_device_lost_callback<F>(&mut self, f: F)
    where F: Fn(wgpu::DeviceLostReason) + Send + 'static  {
        self.device.set_device_lost_callback(move |reason, _| (f)(reason));
    }

    /// Check whether a render pass is currently active. While one is active, writing commands to a command encoder is impossible. 
    pub fn render_pass_active(&mut self) -> bool {
        self.render_pass.is_some()
    }

    fn grab_next_surface(&mut self) {
        self.surface_texture = None;


        let surface = match self.surface.as_ref() {
            Some(surf) => surf,
            None => return
        };

        let mut retry = true;
        let mut retry_available = true;

        while retry {
            retry = false;

            match surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Lost => { return }
                wgpu::CurrentSurfaceTexture::Occluded => {},
                wgpu::CurrentSurfaceTexture::Timeout => {},
                wgpu::CurrentSurfaceTexture::Validation => {},
                wgpu::CurrentSurfaceTexture::Outdated => {
                    surface.configure(&self.device, &self.surface_conf);

                    if retry_available {
                        (retry, retry_available) = (true, false);
                    }
                },
                wgpu::CurrentSurfaceTexture::Suboptimal(surf) => {
                    // A simple trick to make sure that we do reconfigure our surface at the end of the frame
                    self.new_surface_conf = Some(self.surface_conf.clone());

                    self.surface_texture = Some(surf);
                },
                wgpu::CurrentSurfaceTexture::Success(surf) => {
                    self.surface_texture = Some(surf)
                },
            }
        }
    }

    /// Reconfigure our surface and grab the next available texture.
    /// 
    /// Should be called at the end of each frame.
    fn update_and_grab_surface(&mut self) {
        self.surface_texture = None;

        // Apply our new surface configuration
        if let Some(surface_conf) = self.new_surface_conf.take() {
            self.surface_conf = surface_conf;
        }

        if let Some(surf) = self.surface.as_ref() {
            surf.configure(&self.device, &self.surface_conf);
        }

        self.grab_next_surface();
    }

    /// Request to resize the surface at the end of the frame
    pub fn resize_surface(&mut self, width: u32, height: u32) {
        let surface_conf = self.new_surface_conf.get_or_insert(self.surface_conf.clone());

        surface_conf.width = width;
        surface_conf.height = height;
    }

    /// Put a render pass
    fn set_render_pass(&mut self, rpass: wgpu::RenderPass<'static>) {
        self.render_pass.replace(rpass);
    }

    fn drop_render_pass(&mut self) {
        let _ = self.render_pass.take();
    }

    /// Get a command encoder to write commands to (will lazily create one if one doesn't exist)
    fn with_command_encoder<F>(&mut self, f: F)
    where F: FnOnce(&mut wgpu::CommandEncoder) {
        
        // Don't do anything, we currently have a render pass active
        if self.render_pass_active() {
            return;
        }

        let encoder = self.command_encoder.get_or_insert(
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default())
        );

        f(encoder);
    }

    /// Submit any work that was written to our command encoder
    /// 
    /// Usually you only need to call this if you have some dependencies and need results now (for example writing and then reading from a buffer).
    /// By default there's no need to call this, since `commit_frame` does everything automatically.
    pub fn submit_commands(&mut self) {
        self.drop_render_pass();
        
        if let Some(encoder) = self.command_encoder.take() {
            self.queue.submit([encoder.finish()]);
        } 
    }

    pub(crate) fn objects(&self) -> &ObjectStorage {
        &self.objects
    }

    /// Create a new buffer from the provided size, usage and optional mapping parameter
    pub fn new_buffer(&mut self, size: u64, usage: wgpu::BufferUsages, mapped_at_creation: bool) -> BufferId {
        self.objects.new_buffer(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage,
            mapped_at_creation
        }))
    }

    /// Create a new buffer with the provided contents
    pub fn new_buffer_data(&mut self, contents: &[u8], usage: wgpu::BufferUsages) -> BufferId {
        self.objects.new_buffer(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents,
            usage
        }))
    }

    /// Create a new shader module
    pub fn new_shader(&mut self, wgsl: &str) -> ShaderId {
        self.objects.new_shader(
            self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(wgsl))
            })
        )
    }

    /// Create a new texture with the provided parameters
    /// 
    /// - `size` = the dimensions of a texture (depending on a dimension it could mean different things)
    /// - `dimension` = how to interpret the size (A single 2D texture, an array of 2D textures, a cube or a 3D texture)
    /// - `mip_level_count` = how many mip layers this texture has (leave at 1 if there's none)
    /// - `sample_count` = whether the texture is multisampled (leave at 1 if not)
    pub fn new_texture(
        &mut self, 
        size: wgpu::Extent3d, 
        dimension: wgpu::TextureDimension,
        mip_level_count: u32,
        sample_count: u32, 
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages
    ) -> TextureId {
        self.objects.new_texture(self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count,
            sample_count,
            dimension,
            format,
            usage,
            view_formats: &[format]
        }))
    }

    /// A simplified version of [PalCtx::new_texture] for 2D textures
    pub fn new_texture2d(
        &mut self, 
        size: (u32, u32), 
        mip_level_count: u32,
        sample_count: u32, 
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages
    ) -> TextureId {
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
        &mut self, 
        size: (u32, u32, u32),
        mip_level_count: u32, 
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages
    ) -> TextureId {
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
        &mut self, 
        size: (u32, u32),
        layers: u32, 
        mip_level_count: u32, 
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages
    ) -> TextureId {
        self.new_texture(
            wgpu::Extent3d{ width: size.0, height: size.1, depth_or_array_layers: layers}, 
            wgpu::TextureDimension::D2, 
            mip_level_count, 
            1, 
            format, 
            usage
        )
    }

    /// Get the current surface texture 
    pub fn get_surface_texture(&mut self) -> TextureId {
        self.surface_id
    }

    /// Create a new bind group layout
    pub fn new_bind_group_layout(&mut self, layout: &BindGroupLayoutEntries) -> wgpu::BindGroupLayout {
        self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: layout.build()
        })
    }

    /// Create a new pipeline layout 
    pub fn new_pipeline_layout(&mut self, layouts: &[Option<&wgpu::BindGroupLayout>]) -> wgpu::PipelineLayout {
        self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: layouts,
            immediate_size: 0
        })
    }

    /// Create a ne render pipeline from the provided shader, layout, vertex buffer layouts and primitive state
    pub fn new_render_pipeline(
        &mut self, 
        shader: &wgpu::ShaderModule, 
        layout: &wgpu::PipelineLayout,
        vertex_buffer_layouts: &VertexBufferLayouts,
        pipeline_config: &RenderPipelineConfig
    ) -> RPipelineId {
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

        self.objects.new_rpipeline(self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
        }))
    }

    /// Write to a provided buffer with the provided offset
    pub fn write_buffer(&mut self, buffer: BufferId, offset: u64, data: &[u8]) {
        let buffer = match self.objects.get_buffer(buffer) { 
            Some(b) => b,
            None => return
        };

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
        &mut self, 
        texture: TextureId, 
        data: &[u8], 
        mip_level: Option<u32>,
        aspect: Option<wgpu::TextureAspect>,
        origin: wgpu::Origin3d,
        size: wgpu::Extent3d,
    ) {
        let texture = match self.objects.get_texture(texture) { 
            Some(t) => t,
            None => return
        };

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
        &mut self, 
        texture: TextureId, 
        data: &[u8], 
        mip_level: Option<u32>,
        aspect: Option<wgpu::TextureAspect>,
    ) {
        let size = match self.objects.get_texture(texture) {
            Some(texture) => texture.size(),
            None => return
        };

        self.write_texture(texture, data, mip_level, aspect, wgpu::Origin3d::ZERO, size);
    }

    /// Begin a new render pass
    /// 
    /// It's important that render pass 
    pub fn begin_render_pass(&mut self, rpass_builder: &mut RenderPassBuilder) {
        // Make sure that we first drop our existing render pass if one existed
        self.drop_render_pass();

        let mut rpass = None;
        self.with_command_encoder(|enc| {
            rpass = Some(rpass_builder.build_rpass(enc).forget_lifetime());
        });

        // And now replace it
        if let Some(rpass) = rpass {
            self.set_render_pass(rpass);
        }
    }

    /// End and submit the current render pass (if one existed)
    pub fn end_render_pass(&mut self) {
        self.drop_render_pass();
    }

    /// Render the current frame and perform polling
    pub fn commit_frame(&mut self) {
        self.submit_commands();
        
        if let Some(surf) = self.surface_texture.take() {
            self.queue.present(surf);
        }
        
        let _ = self.device.poll(wgpu::PollType::Poll);

        // Perform any configuration and grab our next surface
        self.update_and_grab_surface();
    }
}

impl Drop for PalCtx {
    fn drop(&mut self) {
        // Make sure that our surface texture drops first
        self.surface_texture = None;
        self.command_encoder = None;
        self.render_pass = None;
    }
}

/// Initialize a new PalGL context
/// 
/// # Safety
/// This constructor is insafe because of arbitrary window/display handles handed to that context. 
pub unsafe fn new_palgl_ctx(init: InitContext, conf: &PalCtxConfig) -> PalCtx {
    unsafe { pollster::block_on(PalCtx::new(init, conf)) }
}