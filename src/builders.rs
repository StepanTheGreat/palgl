use std::num::{NonZeroU32, NonZeroU64};

use crate::{ctx::PalCtx, object::TextureId, values::{BlendState, BufferBindingType, CompareFunction, DepthWriteEnabled, IndexFormat, StoreOp, TextureSampleType}};

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

pub struct VertexBufferLayoutBuilder {
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

    pub(crate) fn build<'a>(&'a self) -> Vec<Option<wgpu::VertexBufferLayout<'a>>> {
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

    pub(crate) fn build(&self) -> &[wgpu::BindGroupLayoutEntry] {
        &self.layouts
    }
}

struct ColorAttachmentEntry {
    view: wgpu::TextureView,
    resolve_target: Option<wgpu::TextureView>,
    ops: wgpu::Operations<wgpu::Color>
}

struct DepthAttachment {
    view: wgpu::TextureView,
    depth_ops: Option<wgpu::Operations<f32>>,
    stencil_ops: Option<wgpu::Operations<u32>>
}

pub struct RenderPassBuilder {
    color_attachments: Vec<Option<ColorAttachmentEntry>>,
    depth_attachment: Option<DepthAttachment>,

    color_ops: Option<wgpu::Operations<wgpu::Color>>,
    color_resolve_target: Option<wgpu::TextureView>,

    depth_ops: Option<wgpu::Operations<f32>>,
    stencil_ops: Option<wgpu::Operations<u32>>,
}

impl RenderPassBuilder {
    pub fn new() -> Self {
        Self {
            color_attachments: Vec::new(),
            depth_attachment: None,

            color_resolve_target: None,
            color_ops: None,
            depth_ops: None,
            stencil_ops: None,
        }
    }

    /// Set next color operation before pushing a color attachment
    pub fn set_color_op(&mut self, clear: Option<&[f64; 4]>, store: StoreOp) {

        let load = match clear {
            Some([r, g, b, a]) => wgpu::LoadOp::Clear(wgpu::Color { r: *r, g: *g, b: *b, a: *a }),
            None => wgpu::LoadOp::Load
        };

        self.color_ops = Some(wgpu::Operations { load, store: store.into() });
    }

    /// Set depth operation to use (before setting depth stencil attachment)
    pub fn set_depth_op(&mut self, clear: Option<&f32>, store: StoreOp) {
        let load = match clear {
            Some(clear) => wgpu::LoadOp::Clear(*clear),
            None => wgpu::LoadOp::Load
        };

        self.depth_ops = Some(wgpu::Operations { load, store: store.into()})
    }

    /// Set stencil operation to use (before setting depth stencil attachment)
    pub fn set_stencil_op(&mut self, clear: Option<&u32>, store: StoreOp) {
        let load = match clear {
            Some(clear) => wgpu::LoadOp::Clear(*clear),
            None => wgpu::LoadOp::Load
        };

        self.stencil_ops = Some(wgpu::Operations { load, store: store.into()})
    }

    /// Add a resolve target for the next multisampled color target
    pub fn set_color_resolve_target(&mut self, ctx: &PalCtx, texture: TextureId) {
        if let Some(view) = ctx.objects().get_texture_view(texture) {
            self.color_resolve_target = Some(view);
        }
    }

    /// Push a color attachment with the selected color operation and resolve target
    pub fn push_color_attachment(&mut self, ctx: &PalCtx, view: TextureId) {    
        
        if let Some(view) = ctx.objects().get_texture_view(view) {
            self.color_attachments.push(Some(ColorAttachmentEntry {
                view,
                ops: self.color_ops.take().unwrap_or_default(),
                resolve_target: self.color_resolve_target.take(),
            }));
        }

        self.color_ops = None;
        self.color_resolve_target = None;
    }

    /// Push an empty color attachment
    pub fn push_color_attachment_void(&mut self) {
        self.color_attachments.push(None);

        self.color_ops = None;
        self.color_resolve_target = None;

    }

    pub fn set_depth_stencil_attachment(&mut self, ctx: &PalCtx, view: TextureId) {
        if let Some(view) = ctx.objects().get_texture_view(view) {
            self.depth_attachment = Some(DepthAttachment {
                view: view,
                depth_ops: self.depth_ops,
                stencil_ops: self.stencil_ops
            });
        }

        self.depth_ops = None;
        self.stencil_ops = None;
    }

    /// Clear the entire structure
    pub fn clear(&mut self) {
        self.color_attachments.clear();
        self.depth_attachment = None;

        self.color_ops = None;
        self.color_resolve_target = None;
        self.depth_ops = None;
        self.stencil_ops = None;
    }

    pub(crate) fn build_rpass<'a>(&mut self, encoder: &'a mut wgpu::CommandEncoder) -> wgpu::RenderPass<'a> {
        
        // No safe way to avoid allocations here unfortunately. wgpu inherently requires a slice of texture view references, and self-referential
        // structs are highly unsafe (lifetimes reject them altogether).
        // There is a workaround with pointers (where references are kept as static and then directly casted to 'a when building), but
        // even that has its own unsafety risks. 
        let mut color_attachments = Vec::new();
        let mut depth_attachment = None;

        for color_attachment in self.color_attachments.iter() {

            let color_attachment = 'attachment: {
                let a = match color_attachment {
                    Some(a) => a,
                    None => break 'attachment None
                };
                
                Some(wgpu::RenderPassColorAttachment {
                    view: &a.view,
                    resolve_target: a.resolve_target.as_ref(),
                    ops: a.ops,
                    depth_slice: None
                })
            };

            color_attachments.push(color_attachment);
        }

        if let Some(attachment) = self.depth_attachment.as_ref() {
            depth_attachment = Some(wgpu::RenderPassDepthStencilAttachment {
                view: &attachment.view,
                depth_ops: attachment.depth_ops,
                stencil_ops: attachment.stencil_ops
            });
        }

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &color_attachments,
            depth_stencil_attachment: depth_attachment,
            ..Default::default()
        })
    }
}