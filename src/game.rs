use super::*;

pub fn spawn(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    users: Res<Users>,
) {
    commands.spawn(Camera2d);
    let orb = meshes.add(Circle::new(1.0));
    for user in users.iter() {
        commands.spawn((
            Mesh2d(orb.clone()),
            MeshMaterial2d(mats.add(ColorMaterial::from(server.load_with_settings(
                &user.avatar,
                |s: &mut bevy::image::ImageLoaderSettings| {
                    s.format = bevy::image::ImageFormatSetting::Format(ImageFormat::Jpeg)
                },
            )))),
            Transform::default().with_scale(Vec3::splat(128.)),
        ));
    }
}
