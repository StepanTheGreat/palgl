use std::collections::{HashMap, HashSet, VecDeque};

pub type ObjectId = u32;

/// A simple ID generator and pool for managing object handles.
struct ObjectIDPool {
    counter: ObjectId,
    free: VecDeque<ObjectId>
}

impl ObjectIDPool {
    pub fn new() -> Self {
        // We start with 1, since 0 is considered to be null
        Self {
            counter: 1,
            free: VecDeque::new()
        }
    }

    /// Get a next object ID. It either gets a new counted ID, or uses an already freed one.
    /// 
    /// None means an ID is invalid.
    pub fn take(&mut self) -> Option<ObjectId> {
        if let Some(free_id) = self.free.pop_front() {
            Some(free_id)
        } else {
            let current = self.counter;
        
            if self.counter < u32::MAX {
                self.counter += 1;
                Some(current)
            } else {
                None
            }
        }
    }

    /// Free an existing object ID.
    /// 
    /// This will not check if the object is already freed, so double freeing is possible.
    pub fn free(&mut self, id: ObjectId) {
        if id != 0 {
            self.free.push_back(id);
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct TextureId(ObjectId);

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct BufferId(ObjectId);

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct SamplerId(ObjectId);

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ShaderId(ObjectId);

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct RPipelineId(ObjectId);

/// General object storage
pub(crate) struct ObjectStorage {
    id_pool: ObjectIDPool,

    /// Surface textures are simply reserved handles
    surface_textures: HashMap<ObjectId, ()>,

    textures: HashMap<ObjectId, wgpu::Texture>,
    buffers: HashMap<ObjectId, wgpu::Buffer>,
    samplers: HashMap<ObjectId, wgpu::Sampler>,
    shaders: HashMap<ObjectId, wgpu::ShaderModule>,
    rpipelines: HashMap<ObjectId, wgpu::RenderPipeline>,
}

impl ObjectStorage {
    pub fn new() -> Self {
        Self {
            id_pool: ObjectIDPool::new(),

            surface_textures: HashMap::new(),

            textures: HashMap::new(),
            buffers: HashMap::new(),
            samplers: HashMap::new(),
            shaders: HashMap::new(),
            rpipelines: HashMap::new()
        }
    }

    /// Insert a new object into the provided object set and get a new ID (if possible)
    /// 
    /// If a new ID cannot be allocated - a null pointer is returned
    fn new_object<T>(id_pool: &mut ObjectIDPool, objects: &mut HashMap<ObjectId, T>, object: T) -> ObjectId {
        if let Some(id) = id_pool.take() {
            objects.insert(id, object);
            id
        } else {
            0
        }
    } 

    /// Get an existing object from an existing set if possible
    fn get_object<T: Clone>(objects: &mut HashMap<ObjectId, T>, id: ObjectId) -> Option<T> {
        objects.get(&id).cloned()
    }

    /// Remove an object under provided ID (if present) and free its underlying ID.
    /// 
    /// Returns [Some] if the operation was successful
    fn free_object<T>(id_pool: &mut ObjectIDPool, objects: &mut HashMap<ObjectId, T>, id: ObjectId) -> Option<T> {
        if let Some(obj) = objects.remove(&id) {
            id_pool.free(id);
            Some(obj)
        } else {
            None
        }
    }

    /// Reserve a new surface texture
    /// 
    /// Surface textures are abstract texture handles that map to surfaces, rather than actual textures. They'
    pub fn new_surface_texture(&mut self) -> TextureId {
        TextureId(
            Self::new_object(&mut self.id_pool, &mut self.surface_textures, ())
        )
    }

    pub fn new_texture(&mut self, texture: wgpu::Texture) -> TextureId {
        TextureId(
            Self::new_object(&mut self.id_pool, &mut self.textures, texture)
        )
    }

    pub fn new_buffer(&mut self, buffer: wgpu::Buffer) -> BufferId {
        BufferId(
            Self::new_object(&mut self.id_pool, &mut self.buffers, buffer)
        )
    }

    pub fn new_sampler(&mut self, sampler: wgpu::Sampler) -> SamplerId {
        SamplerId(
            Self::new_object(&mut self.id_pool, &mut self.samplers, sampler)
        )
    }

    pub fn new_shader(&mut self, shader: wgpu::ShaderModule) -> ShaderId {
        ShaderId(
            Self::new_object(&mut self.id_pool, &mut self.shaders, shader)
        )
    }

    pub fn new_rpipeline(&mut self, pipeline: wgpu::RenderPipeline) -> RPipelineId {
        RPipelineId(
            Self::new_object(&mut self.id_pool, &mut self.rpipelines, pipeline)
        )
    }

    pub fn get_texture(&self, id: TextureId) -> Option<&wgpu::Texture> {
        self.textures.get(&id.0)
    }

    pub fn get_texture_view(&self, id: TextureId) -> Option<wgpu::TextureView> {
        self.textures.get(&id.0).map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()))
    }

    pub fn get_buffer(&self, id: BufferId) -> Option<&wgpu::Buffer> {
        self.buffers.get(&id.0)
    }

    pub fn get_sampler(&self, id: SamplerId) -> Option<&wgpu::Sampler> {
        self.samplers.get(&id.0)
    }

    pub fn get_shader(&self, id: ShaderId) -> Option<&wgpu::ShaderModule> {
        self.shaders.get(&id.0)
    }

    pub fn get_rpipeline(&self, id: RPipelineId) -> Option<&wgpu::RenderPipeline> {
        self.rpipelines.get(&id.0)
    }

    pub fn free_texture(&mut self, id: TextureId) -> Option<wgpu::Texture> {
        Self::free_object(&mut self.id_pool, &mut self.textures, id.0)
    }

    pub fn free_buffer(&mut self, id: BufferId) -> Option<wgpu::Buffer> {
        Self::free_object(&mut self.id_pool, &mut self.buffers, id.0)
    }

    pub fn free_sampler(&mut self, id: SamplerId) -> Option<wgpu::Sampler> {
        Self::free_object(&mut self.id_pool, &mut self.samplers, id.0)
    }

    pub fn free_shader(&mut self, id: SamplerId) -> Option<wgpu::ShaderModule> {
        Self::free_object(&mut self.id_pool, &mut self.shaders, id.0)
    }

    pub fn free_rpipeline(&mut self, id: SamplerId) -> Option<wgpu::RenderPipeline> {
        Self::free_object(&mut self.id_pool, &mut self.rpipelines, id.0)
    }
    
    pub fn  free_surface_texture(&mut self, id: TextureId) {
        Self::free_object(&mut self.id_pool, &mut self.surface_textures, id.0);
    }
}