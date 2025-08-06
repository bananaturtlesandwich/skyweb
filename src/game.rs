use super::*;
use avian2d::prelude::*;

const RADIUS: f32 = 20.0;

pub fn spawn(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    users: Res<Users>,
    windows: Query<&Window>,
) {
    commands.spawn(Camera2d);
    let window = windows.single().unwrap();
    // don't want our orbs escaping containment
    commands.spawn((
        Collider::half_space(Vec2::Y),
        RigidBody::Static,
        Transform::from_translation(Vec3::NEG_Y * window.height() / 2.0),
    ));
    let orb = meshes.add(Circle::new(RADIUS));
    let circle = Collider::circle(RADIUS);
    for (i, user) in users.iter().enumerate() {
        commands.spawn((
            circle.clone(),
            RigidBody::Dynamic,
            LinearVelocity(Vec2::NEG_Y * 10.0),
            AngularVelocity(0.1),
            Mesh2d(orb.clone()),
            MeshMaterial2d(mats.add(ColorMaterial::from(server.load_with_settings(
                &user.avatar,
                |s: &mut bevy::image::ImageLoaderSettings| {
                    s.format = bevy::image::ImageFormatSetting::Format(ImageFormat::Jpeg)
                },
            )))),
            user.clone(),
            Transform::from_translation(Vec3::Y * (RADIUS + 10.0) * i as f32),
        ));
    }
    commands.remove_resource::<Users>();
}

pub fn link(trigger: Trigger<Pointer<Pressed>>, users: Query<&User>) {
    let Ok(user) = users.get(trigger.target()) else {
        return;
    };
    webbrowser::open(&format!("https://bsky.app/profile/{}", user.handle));
}
