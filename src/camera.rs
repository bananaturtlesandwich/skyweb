use super::*;

pub struct Stuff;

impl Plugin for Stuff {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                zoom,
                pan.run_if(bevy::input::common_conditions::input_pressed(
                    // really should be PointerButton::Secondary but those don't seem to be the same
                    MouseButton::Right,
                )),
            )
                .run_if(in_state(Game::Connect)),
        );
    }
}

fn zoom(
    mut mouse: EventReader<bevy::input::mouse::MouseWheel>,
    time: Res<Time>,
    config: Res<Config>,
    mut proj: Single<&mut Projection>,
) {
    let Projection::Orthographic(proj) = &mut **proj else {
        return;
    };
    for scroll in mouse.read() {
        proj.scale += scroll.y * config.zoom * time.delta_secs();
    }
}

fn pan(
    mut mouse: EventReader<bevy::input::mouse::MouseMotion>,
    time: Res<Time>,
    config: Res<Config>,
    proj: Single<&Projection>,
    mut trans: Single<&mut Transform, With<Camera2d>>,
) {
    let Projection::Orthographic(proj) = &**proj else {
        return;
    };
    for motion in mouse.read() {
        trans.translation.x -= motion.delta.x * config.pan * proj.scale * time.delta_secs();
        trans.translation.y += motion.delta.y * config.pan * proj.scale * time.delta_secs();
    }
}
