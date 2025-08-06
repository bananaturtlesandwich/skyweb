use super::*;
use avian2d::prelude::*;

const ATTRACTION: f32 = 50.0;

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
    let width = window.width();
    let height = window.height();
    // don't want our orbs escaping containment
    commands.spawn((
        Collider::half_space(Vec2::Y),
        RigidBody::Static,
        Transform::from_translation(Vec3::NEG_Y * height / 2.0),
    ));
    commands.spawn((
        Collider::half_space(Vec2::NEG_Y),
        RigidBody::Static,
        Transform::from_translation(Vec3::Y * height / 2.0),
    ));
    commands.spawn((
        Collider::half_space(Vec2::X),
        RigidBody::Static,
        Transform::from_translation(Vec3::NEG_X * width / 2.0),
    ));
    commands.spawn((
        Collider::half_space(Vec2::NEG_X),
        RigidBody::Static,
        Transform::from_translation(Vec3::X * width / 2.0),
    ));
    let radius = (width * height / users.len() as f32 / std::f32::consts::PI).sqrt() / 2.0;
    let orb = meshes.add(Circle::new(radius));
    let circle = Collider::circle(radius);
    let mut pos = Vec3::new(-width / 2.0 + radius, -height / 2.0 + radius, 0.0);
    let entities = users
        .iter()
        .map(|user| {
            let ent = commands
                .spawn((
                    circle.clone(),
                    RigidBody::Dynamic,
                    AngularVelocity(0.1),
                    Mesh2d(orb.clone()),
                    MeshMaterial2d(mats.add(ColorMaterial::from(server.load_with_settings(
                        &user.avatar,
                        |s: &mut bevy::image::ImageLoaderSettings| {
                            s.format = bevy::image::ImageFormatSetting::Format(ImageFormat::Jpeg)
                        },
                    )))),
                    Transform::from_translation(pos),
                ))
                .id();
            pos.x += radius * 2.0;
            if pos.x > width / 2.0 {
                pos.x -= width;
                pos.y += radius * 2.0;
            }
            ent
        })
        .collect::<Vec<_>>();
    for (user, ent) in users.iter().zip(&entities) {
        commands.entity(*ent).insert(UserComp {
            handle: user.handle.clone(),
            shared: user.shared.iter().map(|i| entities[*i]).collect(),
        });
    }
    commands.remove_resource::<Users>();
}

pub fn attract(
    mut gizmo: Gizmos,
    time: Res<Time>,
    mut users: Query<(Entity, &UserComp, &mut LinearVelocity)>,
    transforms: Query<&Transform>,
) {
    for (ent, user, mut vel) in &mut users {
        for shared in &user.shared {
            let Ok(trans) = transforms.get(ent) else {
                continue;
            };
            let Ok(shared) = transforms.get(*shared) else {
                continue;
            };
            gizmo.line(
                trans.translation.with_z(-1.0),
                shared.translation.with_z(-1.0),
                LinearRgba::GREEN,
            );
            vel.0 += (shared.translation.xz() - trans.translation.xz()).normalize()
                * ATTRACTION
                * time.delta_secs();
        }
    }
}

pub fn link(trigger: Trigger<Pointer<Pressed>>, users: Query<&UserComp>) {
    let Ok(user) = users.get(trigger.target()) else {
        return;
    };
    webbrowser::open(&format!("https://bsky.app/profile/{}", user.handle)).unwrap();
}
