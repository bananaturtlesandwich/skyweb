use super::*;

pub struct Stuff;

impl Plugin for Stuff {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            pan.run_if(bevy::input::common_conditions::input_pressed(
                MouseButton::Middle,
            ))
            .run_if(in_state(Game::Connect)),
        );
    }
}

fn pan(
    mut mouse: EventReader<bevy::input::mouse::MouseMotion>,
    proj: Single<&Projection>,
    mut trans: Single<&mut Transform, With<Camera2d>>,
) {
    let Projection::Orthographic(proj) = &**proj else {
        return;
    };
    for motion in mouse.read() {
        trans.translation.x -= motion.delta.x * proj.scale;
        trans.translation.y += motion.delta.y * proj.scale;
    }
}
