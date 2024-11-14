use vide::prelude::Clip;
use vide_project::Project;

fn main() {
    let mut project = Project::new();

    let clip = Clip::new(0.0..5.0);
    project.add_clip(clip);
}
