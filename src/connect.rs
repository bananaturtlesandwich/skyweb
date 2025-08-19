use super::*;

pub struct Stuff;

impl Plugin for Stuff {
    fn build(&self, app: &mut App) {
        app.register_type::<Config>()
            .init_resource::<Config>()
            .add_systems(OnEnter(Game::Connect), setup)
            .add_systems(Update, (connect, web).run_if(in_state(Game::Connect)))
            .add_observer(link);
    }
}

fn setup(mut commands: Commands, res: Res<Users>, users: Query<(Entity, &User, &Transform)>) {
    let last = res.len() - 1;
    let nodes: Vec<_> = users
        .iter()
        .sort_unstable_by_key::<&User, _>(|user: &&User| user.index)
        .map(|(_, user, trans)| match user.index == last {
            true => fjadra::Node::default()
                .fixed_position(trans.translation.x as f64, trans.translation.y as f64),
            false => fjadra::Node::default()
                .position(trans.translation.x as f64, trans.translation.y as f64),
        })
        .collect();
    let links: Vec<_> = users
        .iter()
        .filter(|(_, user, _)| user.index != last)
        .flat_map(|(_, user, _)| {
            user.shared
                .iter()
                .filter_map(|ent| Some((user.index, users.get(*ent).ok()?.1.index)))
        })
        .collect();
    commands.insert_resource(Sim {
        sim: fjadra::SimulationBuilder::new()
            .build(nodes.iter().cloned())
            .add_force(
                "link",
                fjadra::Link::new(links.iter().cloned()).strength(10.0),
            )
            .add_force("charge", fjadra::ManyBody::new().strength(-1500.0))
            .add_force("centre", fjadra::Center::new()),
        nodes,
        links,
    });
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
