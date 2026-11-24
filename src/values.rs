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

#[repr(u32)]
pub enum StoreOp {
    /// Stores the resulting value of the render pass for this attachment.
    Store = 1,
    
    /// Discards the resulting value of the render pass for this attachment.
    ///
    /// The attachment will be treated as uninitialized afterwards.
    /// (If only either Depth or Stencil texture-aspects is set to `Discard`,
    /// the respective other texture-aspect will be preserved.)
    ///
    /// This can be significantly faster on tile-based render hardware.
    ///
    /// Prefer this if the attachment is not read by subsequent passes.
    Discard = 2,
}

impl From<StoreOp> for wgpu::StoreOp {
    fn from(value: StoreOp) -> Self {
        match value {
            StoreOp::Store => Self::Store,
            StoreOp::Discard => Self::Discard
        }
    }
}