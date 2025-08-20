use super::*;

pub struct Stuff;

impl Plugin for Stuff {
    fn build(&self, app: &mut App) {
        app.register_type::<Config>()
            .init_resource::<Config>()
            .add_systems(OnEnter(Game::Connect), setup)
            .add_systems(Update, (connect, web).run_if(in_state(Game::Connect)))
            .add_observer(rebuild)
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
    config: Res<Config>,
    mut sim: ResMut<Sim>,
    users: Query<(&Transform, &User)>,
) {
    for (node, (trans, _)) in sim
        .nodes
        .iter_mut()
        .zip(users.iter().sort_by_key::<&User, usize>(|user| user.index))
    {
        // this doesn't reset fixed
        *node =
            std::mem::take(node).position(trans.translation.x as f64, trans.translation.y as f64);
    }
    let mut link = fjadra::Link::new(sim.links.iter().cloned()).distance(config.distance);
    if let Some(slink) = config.link {
        link = link.strength(slink)
    }
    **sim = fjadra::SimulationBuilder::new()
        .build(sim.nodes.iter().cloned())
        .add_force("link", link)
        .add_force("charge", fjadra::ManyBody::new().strength(config.charge))
        .add_force("centre", fjadra::Center::new().strength(config.centre));
}

fn connect(mut sim: ResMut<Sim>, stats: Res<Config>, mut users: Query<(&User, &mut Transform)>) {
    sim.tick(stats.iter);
    for ([x, y], (_, mut trans)) in sim.positions().zip(
        users
            .iter_mut()
            .sort_unstable_by_key::<&User, _>(|user: &&User| user.index),
    ) {
        trans.translation.x = x as f32;
        trans.translation.y = y as f32;
    }
}

fn link(trigger: Trigger<Pointer<Pressed>>, mut ctx: bevy_egui::EguiContexts, users: Query<&User>) {
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
    interactions: Query<&bevy::picking::pointer::PointerInteraction>,
    users: Query<(&User, &Transform)>,
) {
    for (ent, _) in interactions
        .iter()
        .filter_map(bevy::picking::pointer::PointerInteraction::get_nearest_hit)
    {
        let Ok((user, trans)) = users.get(*ent) else {
            continue;
        };
        for shared in &user.shared {
            let Ok((user2, trans2)) = users.get(*shared) else {
                continue;
            };
            gizmo.line(
                trans.translation,
                trans2.translation,
                match user2.shared.contains(&ent) {
                    true => LinearRgba::GREEN,
                    false => LinearRgba::BLUE,
                },
            )
        }
    }
}
