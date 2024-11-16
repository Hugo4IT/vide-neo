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
        }
    }

    fn create_texture(&self, wgpu: &Wgpu) -> FactoryTexture {
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
}
