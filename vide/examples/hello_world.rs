use std::path::Path;

use euler::vec2;
use vide::prelude::*;
use vide_ffmpeg::MediaExporter;
use vide_project::Project;
#[allow(unused_imports)]
use vide_render::export::{gif::GifExporter, images::ImageExporter};
use vide_video::rect_shape::RectShape;

fn main() {
    let mut project = Project::new();

    let mut clip = Clip::new(0.0..5.0);

    clip.attach_video(RectShape {
        position: animated(vec2!(0.0))
            .keyframe_ease(Abs(Seconds(1.0)), vec2!(400.0, 200.0), EASE_IN_OUT_QUART)
            .hold(Seconds(1.0))
            .keyframe_ease(Rel(Seconds(2.0)), vec2!(200, 400.0), EASE_OUT_EXPO)
            .build(),
        rotation: animated(0.0)
            .hold(Seconds(2.0))
            .keyframe_ease(Rel(Seconds(2.0)), 360.0 * 3.0, EASE_OUT_EXPO)
            .build(),
        size: value(vec2!(100.0)),
        pivot: value(vec2!(0.0)),
        color: animated("#da0037")
            .keyframe(Abs(Seconds(1.0)), "#37da00")
            .hold(Seconds(1.0))
            .keyframe(Rel(Seconds(2.0)), Color::WHITE)
            .build(),
        ..Default::default()
    });

    project.add_clip(clip);

    render(
        project,
        RenderConfiguration {
            resolution: RESOLUTION_1080P_16X9,
            frames_per_second: FPS_60,
            hdr: false,
        },
        MediaExporter::new(Path::new("test-output/vide.mp4")),
        // ImageExporter::new(|frame| Path::new(&format!("test-output/{frame:04}.png")).to_path_buf()),
    );
}
