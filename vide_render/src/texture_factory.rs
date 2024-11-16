use vide_common::render::Wgpu;

#[derive(Debug)]
pub struct FactoryTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl FactoryTexture {
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

#[derive(Debug)]
pub struct TextureFactory {
    slots: Vec<FactoryTexture>,
    descriptor: wgpu::TextureDescriptor<'static>,
    view_descriptor: wgpu::TextureViewDescriptor<'static>,
    created_textures: usize,
}

impl TextureFactory {
    pub fn new(
        descriptor: wgpu::TextureDescriptor<'static>,
        view_descriptor: wgpu::TextureViewDescriptor<'static>,
    ) -> Self {
        Self {
            slots: Vec::new(),
            descriptor,
            view_descriptor,
            created_textures: 0,
        }
    }

    fn create_texture(&mut self, wgpu: &Wgpu) -> FactoryTexture {
        self.created_textures += 1;

        log::info!("Creating texture #{}", self.created_textures);

        let texture = wgpu.device.create_texture(&self.descriptor);
        let view = texture.create_view(&self.view_descriptor);

        FactoryTexture { texture, view }
    }

    pub fn borrow_texture(&mut self, wgpu: &Wgpu) -> FactoryTexture {
        if let Some(slot) = self.slots.pop() {
            slot
        } else {
            self.create_texture(wgpu)
        }
    }

    pub fn return_texture(&mut self, texture: FactoryTexture) {
        self.slots.push(texture);
    }

    pub fn created_textures(&self) -> usize {
        self.created_textures
    }

    pub fn available_textures(&self) -> usize {
        self.slots.len()
    }
}
