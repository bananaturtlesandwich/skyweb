use super::*;

pub struct Stuff;

impl Plugin for Stuff {
    fn build(&self, app: &mut App) {
        app.add_systems(
            bevy_egui::EguiPrimaryContextPass,
            config.run_if(in_state(Game::Connect)),
        );
    }
}

fn config(
    mut ctx: bevy_egui::EguiContexts,
    mut commands: Commands,
    mut config: ResMut<Config>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut next: ResMut<NextState<Game>>,
    orb: Res<Orb>,
    users: Query<Entity, With<User>>,
    mut proj: Single<&mut Projection>,
) {
    use bevy_egui::egui;
    let Ok(ctx) = ctx.ctx_mut() else { return };
    let Projection::Orthographic(proj) = &mut **proj else {
        return;
    };
    #[rustfmt::skip]
    // on wasm this is shown on the webpage
    #[cfg(not(target_family = "wasm"))]
    egui::Window::new("about").default_open(false).show(ctx, |ui| {
        ui.label("skyweb is a fun thing i made because i randomly went down the atproto webtoy rabbit hole and wanted to make one!");
        ui.spacing_mut().item_spacing.x = ui.fonts(|fonts| fonts.glyph_width(&egui::TextStyle::Body.resolve(ui.style()), ' '));
        ui.horizontal_wrapped(|ui| {
            ui.label("if you like this consider buying me a");
            ui.hyperlink_to("kofi", "https://ko-fi.com/bananaturtlesandwich");
            ui.label(", following me on");
            ui.hyperlink_to("bsky", "https://bsky.app/profile/spuds.casa");
            ui.label("or checking out ");
            ui.hyperlink_to("the rest of my website", "https://spuds.casa");
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("you can also check out the code over on ");
            ui.hyperlink_to("github", "https://github.com/bananaturtlesandwich/skyweb");
        });
        ui.label("this was made possible by:");
        ui.horizontal_wrapped(|ui| {
            ui.label("•");
            ui.hyperlink_to("atrium", "https://crates.io/crates/atrium-api");
            ui.label("which is the wonderful rust atproto implementation")
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("•");
            ui.hyperlink_to("bevy", "https://bevy.org");
            ui.label("which handles all the windowing, asset loading, state and ecs")
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("•");
            ui.hyperlink_to("fjadra", "https://crates.io/crates/fjadra");
            ui.label("which implements the verlet physics for laying out the orbs")
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("•");
            ui.hyperlink_to("bevy egui", "https://crates.io/crates/bevy_egui");
            ui.label("which was able to fill in for bevy while its widgets are still cooking")
        });
    });
    let mut rebuild = false;
    #[rustfmt::skip]
    egui::Window::new("config").show(ctx, |ui| {
        ui.label("to pan the camera");
        ui.label("right-click + drag");
        ui.horizontal(|ui| {
            ui.label("paused:");
            ui.checkbox(&mut config.paused, egui::Atoms::default());
        });
        ui.horizontal(|ui| {
            ui.label("speed:");
            ui.add(egui::DragValue::new(&mut config.speed).range(1..=usize::MAX))
        });
        ui.horizontal(|ui| {
            ui.label("charge:");
            rebuild |= ui.add(egui::DragValue::new(&mut config.charge).range(f32::MIN..=0.0)).changed();
        });
        ui.horizontal(|ui| {
            ui.label("link:");
            rebuild |= ui.add(egui::DragValue::new(&mut config.link).range(0.0..=f32::MAX)).changed();
        });
        if rebuild {
            commands.trigger(Rebuild)
        }
        ui.horizontal(|ui| {
            ui.label("size:");
            if ui.add(egui::DragValue::new(&mut config.size).range(0.0..=f32::MAX)).changed() {
                if let Some(orb) = meshes.get_mut(&**orb) {
                    *orb = Mesh::from(Circle::new(config.size))
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("zoom:");
            ui.add(egui::DragValue::new(&mut proj.scale).range(0.1..=f32::MAX).speed(0.02));
        });
        if ui.button("reset").clicked() {
            commands.remove_resource::<Sim>();
            commands.remove_resource::<Profile>();
            commands.remove_resource::<Network>();
            commands.remove_resource::<Lines>();
            for ent in &users {
                commands.entity(ent).despawn()
            }
            next.set(Game::Ask)
        }
    });
}
