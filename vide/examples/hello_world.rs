use euler::vec2;
use vide::prelude::*;
use vide_project::Project;
use vide_video::rect_shape::RectShape;

fn main() {
    let mut project = Project::new();

    let mut clip = Clip::new(0.0..5.0);
    clip.attach_video(RectShape {
        position: animated(vec2!(0.0))
            .keyframe_ease(Abs(Seconds(1.0)), vec2!(400.0, -200.0), EASE_IN_OUT_QUART)
            .hold(Seconds(1.0))
            .keyframe_ease(Rel(Seconds(2.0)), vec2!(200, -400.0), EASE_OUT_EXPO)
            .build(),
        size: value(vec2!(100.0)),
        pivot: value(vec2!(0.5)),
        color: animated("#da0037")
            .keyframe(Abs(Seconds(1.0)), "#37da00")
            .hold(Seconds(1.0))
            .keyframe(Rel(Seconds(2.0)), Color::WHITE)
            .build(),
    });
    project.add_clip(clip);
}
