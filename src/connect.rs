use super::*;

pub struct Stuff;

impl Plugin for Stuff {
    fn build(&self, app: &mut App) {
        app.register_type::<Config>()
            .init_resource::<Config>()
            .add_systems(OnEnter(Game::Connect), (setup, lines))
            .add_systems(Update, (connect, web).run_if(in_state(Game::Connect)))
            .add_observer(rebuild)
            .add_observer(over)
            .add_observer(out)
            .add_observer(link);
    }
}

fn setup(mut commands: Commands, network: Res<Network>) {
    let count = network.len();
    let mut nodes = vec![fjadra::Node::default(); count];
    nodes[count - 1] = fjadra::Node::default().fixed_position(0.0, 0.0);
    commands.insert_resource(Sim {
        sim: fjadra::SimulationBuilder::new()
            .build(nodes.iter().cloned())
            .add_force("link", fjadra::Link::new([]))
            .add_force("charge", fjadra::ManyBody::new())
            .add_force("centre", fjadra::Center::new()),
        nodes,
        links: (0..count).map(|i| (count - 1, i)).collect(),
    });
}

fn rebuild(
    _: Trigger<Rebuild>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sim: ResMut<Sim>,
    config: Res<Config>,
    network: Res<Network>,
    lines: Res<Lines>,
    users: Query<(&Transform, &User)>,
) {
    let Some(mesh) = meshes.get_mut(&**lines) else {
        return;
    };
    mesh.insert_indices(bevy::render::mesh::Indices::U32(
        sim.links
            .iter()
            // don't really care about follows you share with yourself
            .skip(network.len())
            .flat_map(|(i1, i2)| [*i1 as u32, *i2 as u32])
            .collect(),
    ));
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        bevy::render::mesh::VertexAttributeValues::Float32x4(
            users
                .iter()
                .sort_unstable_by_key::<&User, _>(|user: &&User| user.index)
                .map(|(_, user)| {
                    let fraction = user.shared.len() as f64 / network.max as f64;
                    let colour = colorous::PLASMA.eval_continuous(fraction);
                    [
                        colour.r as f32 / 255.0,
                        colour.g as f32 / 255.0,
                        colour.b as f32 / 255.0,
                        fraction as f32,
                    ]
                })
                .collect(),
        ),
    );
    for (node, (trans, _)) in sim
        .nodes
        .iter_mut()
        .zip(users.iter().sort_by_key::<&User, usize>(|user| user.index))
    {
        // this doesn't reset fixed
        *node =
            std::mem::take(node).position(trans.translation.x as f64, trans.translation.y as f64);
    }
    **sim = fjadra::SimulationBuilder::new()
        .build(sim.nodes.iter().cloned())
        .add_force(
            "link",
            fjadra::Link::new(sim.links.iter().cloned()).distance(config.link),
        )
        .add_force("charge", fjadra::ManyBody::new().strength(config.charge))
        .add_force("centre", fjadra::Center::new());
}

fn connect(
    mut sim: ResMut<Sim>,
    mut meshes: ResMut<Assets<Mesh>>,
    config: Res<Config>,
    mut users: Query<(&User, &mut Transform)>,
    lines: Res<Lines>,
) {
    if config.paused || sim.is_finished() {
        return;
    }
    sim.tick(config.speed);
    let Some(mesh) = meshes.get_mut(&**lines) else {
        return;
    };
    let cap = users.iter().count();
    let mut position = Vec::with_capacity(cap);
    for ([x, y], (_, mut trans)) in sim.positions().zip(
        users
            .iter_mut()
            .sort_unstable_by_key::<&User, _>(|user: &&User| user.index),
    ) {
        trans.translation.x = x as f32;
        trans.translation.y = y as f32;
        position.push([x as f32, y as f32, 0.0]);
    }
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        bevy::render::mesh::VertexAttributeValues::Float32x3(position),
    );
}

fn lines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<ColorMaterial>>,
) {
    let lines = meshes.add(Mesh::new(
        bevy::render::mesh::PrimitiveTopology::LineList,
        bevy::asset::RenderAssetUsages::default(),
    ));
    commands.spawn((
        Mesh2d(lines.clone_weak()),
        MeshMaterial2d(mats.add(ColorMaterial::default())),
        Transform::from_translation(Vec3::NEG_Z),
    ));
    commands.insert_resource(Lines(lines));
}

fn over(
    trigger: Trigger<Pointer<Over>>,
    mut commands: Commands,
    window: Single<Entity, With<bevy::window::PrimaryWindow>>,
    users: Query<(), With<User>>,
) {
    if !users.contains(trigger.target()) {
        return;
    }
    commands
        .entity(*window)
        .insert(bevy::winit::cursor::CursorIcon::System(
            bevy::window::SystemCursorIcon::Pointer,
        ));
}

fn out(
    trigger: Trigger<Pointer<Out>>,
    mut commands: Commands,
    window: Single<Entity, With<bevy::window::PrimaryWindow>>,
    users: Query<(), With<User>>,
) {
    if !users.contains(trigger.target()) {
        return;
    }
    commands
        .entity(*window)
        .insert(bevy::winit::cursor::CursorIcon::System(
            bevy::window::SystemCursorIcon::Default,
        ));
}

fn link(trigger: Trigger<Pointer<Pressed>>, mut ctx: bevy_egui::EguiContexts, users: Query<&User>) {
    if trigger.button != PointerButton::Primary {
        return;
    }
    if ctx.ctx_mut().is_ok_and(|ctx| ctx.is_pointer_over_area()) {
        return;
    }
    let Ok(user) = users.get(trigger.target()) else {
        return;
    };
    let _ = webbrowser::open(&format!("https://bsky.app/profile/{}", user.handle));
}

fn web(
    mut gizmo: Gizmos,
    mut ctx: bevy_egui::EguiContexts,
    network: Res<Network>,
    interactions: Query<&bevy::picking::pointer::PointerInteraction>,
    users: Query<(&User, &Transform)>,
    proj: Single<(&Transform, &Projection)>,
) {
    use bevy_egui::egui;
    let Ok(ctx) = ctx.ctx_mut() else { return };
    let (camera, Projection::Orthographic(proj)) = &*proj else {
        return;
    };
    for (ent, _) in interactions
        .iter()
        .filter_map(bevy::picking::pointer::PointerInteraction::get_nearest_hit)
    {
        let Ok((user, trans)) = users.get(*ent) else {
            continue;
        };
        let dim = ctx.available_rect();
        let text = egui::style::default_text_styles()[&egui::TextStyle::Body].clone();
        let pad = ctx.style().spacing.window_margin.bottomf();
        let pos = egui::pos2(
            (trans.translation.x - camera.translation.x) / proj.scale + dim.width() / 2.0,
            (-trans.translation.y + camera.translation.y) / proj.scale + dim.height() / 2.0,
        );
        let mut start = pos.clone();
        start.x -= pad;
        start.y -= pad;
        let mut end = pos.clone();
        end.x += ctx.fonts(|fonts| {
            user.handle
                .chars()
                .fold(0.0, |acc, c| acc + fonts.glyph_width(&text, c))
        }) + pad;
        end.y += text.size + pad;
        ctx.debug_painter().rect_filled(
            egui::Rect::from_min_max(start, end),
            ctx.style().visuals.window_corner_radius,
            ctx.style().visuals.widgets.noninteractive.bg_fill,
        );
        ctx.debug_painter().text(
            pos,
            egui::Align2::LEFT_TOP,
            user.handle.clone(),
            text,
            ctx.style().visuals.noninteractive().text_color(),
        );
        for (ent2, (user2, trans2)) in network
            .iter()
            .filter_map(|(_, ent)| Some((ent, users.get(*ent).ok()?)))
        {
            let follows = user.shared.contains(ent2);
            let followed = user2.shared.contains(ent);
            gizmo.line(
                trans.translation,
                trans2.translation,
                match (follows, followed) {
                    (true, true) => LinearRgba::GREEN,
                    (true, false) => LinearRgba::BLUE,
                    (false, true) => LinearRgba::RED,
                    (false, false) => continue,
                },
            )
        }
    }
}
