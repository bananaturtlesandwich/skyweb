use super::*;
use avian2d::prelude::*;

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
    let mut pos = Vec3::ZERO;
    let mut layer = 0;
    let mut capacity = 1;
    let mut angle = 0.0;
    let mut counter = 0;
    let entities = users
        .iter()
        .map(|user| {
            let ent = commands
                .spawn((
                    circle.clone(),
                    RigidBody::Dynamic,
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
            // it's currently half working
            counter += 1;
            pos = Quat::from_rotation_z(angle) * pos;
            if counter == capacity {
                counter = 0;
                layer += 1;
                // circumference of layer circle is 2*radius*layer*pi
                // orb capacity in each layer is circumference/radius = 2*layer*pi
                let cap = 2.0 * std::f32::consts::PI * layer as f32;
                capacity = cap.floor() as u32;
                // angle to rotate by is 2*pi/capacity = 2*pi / 2*pi*layer = 1/layer
                angle = 1.0 / layer as f32;
                pos += Vec3::Y * radius * 2.5;
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
    stats: Res<Stats>,
    mut users: Query<(Entity, &UserComp, &Transform, &mut LinearVelocity)>,
) {
    let mut combinations = users.iter_combinations_mut();
    while let Some(
        [
            (ent1, user1, trans1, mut vel1),
            (ent2, user2, trans2, mut vel2),
        ],
    ) = combinations.fetch_next()
    {
        let pre =
            (trans2.translation.xy() - trans1.translation.xy()).normalize() * time.delta_secs();
        let attraction = pre * stats.attraction;
        let repulsion = -pre * stats.repulsion;
        let contains1 = user1.shared.contains(&ent2);
        vel1.0 += match contains1 {
            true => attraction,
            false => repulsion,
        };
        let contains2 = user2.shared.contains(&ent1);
        vel2.0 += match contains2 {
            true => -attraction,
            false => -repulsion,
        };
        if contains1 || contains2 {
            gizmo.line(
                trans1.translation,
                trans2.translation,
                match contains1 && contains2 {
                    true => LinearRgba::RED,
                    false => LinearRgba::GREEN,
                },
            );
        }
    }
    for (_, _, trans, mut vel) in &mut users {
        vel.0 -= trans.translation.xy() * stats.gravity;
    }
}

pub fn link(trigger: Trigger<Pointer<Pressed>>, users: Query<&UserComp>) {
    let Ok(user) = users.get(trigger.target()) else {
        return;
    };
    webbrowser::open(&format!("https://bsky.app/profile/{}", user.handle)).unwrap();
}
