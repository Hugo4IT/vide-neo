use vide_common::{
    time_code::UnboundedTimecodeRange, transform::Transform, visible_object::VisibleObject,
};

#[derive(Debug)]
pub struct Clip {
    range: UnboundedTimecodeRange,
    children: Vec<Clip>,
    video_source: Option<Box<dyn VisibleObject>>,
    transform: Transform,
}

impl Clip {
    pub fn new(range: impl Into<UnboundedTimecodeRange>) -> Self {
        Self {
            range: range.into(),
            children: Vec::new(),
            video_source: None,
            transform: Transform::new(),
        }
    }

    pub fn attach_video(&mut self, source: impl VisibleObject + 'static) {
        self.video_source = Some(Box::new(source));
    }

    pub fn infer_duration(&mut self) {
        if let Some(duration) = self.video_source.as_ref().and_then(|v| v.duration()) {
            self.range.set_duration(duration);
        }
    }

    pub fn range(&self) -> UnboundedTimecodeRange {
        UnboundedTimecodeRange::new(
            self.range.start(),
            self.range
                .end()
                .or_else(|| self.children.iter().filter_map(|c| c.range().end()).max()),
        )
    }

    pub fn add_clip(&mut self, clip: Clip) {
        self.children.push(clip);
    }

    pub fn transform(&self) -> Transform {
        self.transform
    }

    pub fn video_mut(&mut self) -> Option<&mut Box<dyn VisibleObject>> {
        self.video_source.as_mut()
    }

    pub fn children_mut(&mut self) -> &mut [Clip] {
        &mut self.children
    }
}
