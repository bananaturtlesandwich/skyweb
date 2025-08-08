use super::*;
use avian2d::prelude::*;

pub struct Stuff;

impl Plugin for Stuff {
    fn build(&self, app: &mut App) {
        app.register_type::<Config>()
            .init_resource::<Config>()
            .add_systems(
                Update,
                (
                    attract.run_if(
                        |stats: Res<Config>, mut timer: Local<Option<Timer>>, time: Res<Time>| {
                            let timer = timer.get_or_insert_with(|| {
                                Timer::from_seconds(stats.tick, TimerMode::Repeating)
                            });
                            if stats.is_changed() {
                                timer.set_duration(std::time::Duration::from_secs_f32(stats.tick));
                            }
                            timer.tick(time.delta());
                            timer.just_finished()
                        },
                    ),
                    web,
                    resize,
                )
                    .run_if(in_state(Game::Connect)),
            )
            .add_observer(link);
    }
}

fn attract(
    time: Res<Time>,
    stats: Res<Config>,
    mut users: Query<(Entity, &User, &Transform, &mut LinearVelocity)>,
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
    }
    for (_, _, trans, mut vel) in &mut users {
        vel.0 -= trans.translation.xy() * stats.gravity;
    }
}

fn resize(
    mut events: EventReader<bevy::window::WindowResized>,
    mut commands: Commands,
    bounds: Query<Entity, (With<Collider>, Without<User>)>,
) {
    for event in events.read() {
        for bound in &bounds {
            commands.entity(bound).despawn();
        }
        commands.spawn((
            Collider::half_space(Vec2::Y),
            RigidBody::Static,
            Transform::from_translation(Vec3::NEG_Y * event.height / 2.0),
        ));
        commands.spawn((
            Collider::half_space(Vec2::NEG_Y),
            RigidBody::Static,
            Transform::from_translation(Vec3::Y * event.height / 2.0),
        ));
        commands.spawn((
            Collider::half_space(Vec2::X),
            RigidBody::Static,
            Transform::from_translation(Vec3::NEG_X * event.width / 2.0),
        ));
        commands.spawn((
            Collider::half_space(Vec2::NEG_X),
            RigidBody::Static,
            Transform::from_translation(Vec3::X * event.width / 2.0),
        ));
    }
}

fn link(trigger: Trigger<Pointer<Pressed>>, mut ctx: bevy_egui::EguiContexts, users: Query<&User>) {
    if ctx.ctx_mut().is_ok_and(|ctx| ctx.is_pointer_over_area()) {
        return;
    }
    let Ok(user) = users.get(trigger.target()) else {
        return;
    };
    webbrowser::open(&format!("https://bsky.app/profile/{}", user.handle)).unwrap();
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
