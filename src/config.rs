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

#[rustfmt::skip]
fn config(mut ctx: bevy_egui::EguiContexts, mut commands:Commands, mut config: ResMut<Config>, mut meshes: ResMut<Assets<Mesh>>, orb: Res<Orb>) {
    use bevy_egui::egui;
    let Ok(ctx) = ctx.ctx_mut() else { return };
    let mut rebuild = false;
    egui::Window::new("config").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("speed:");
            ui.add(egui::DragValue::new(&mut config.speed))
        });
        ui.horizontal(|ui| {
            ui.label("charge:");
            rebuild |= ui.add(egui::DragValue::new(&mut config.charge)).changed();
        });
        ui.horizontal(|ui| {
            ui.label("link:");
            rebuild |= ui.add(egui::DragValue::new(&mut config.link)).changed();
        });
        if rebuild {
            commands.trigger(Rebuild)
        }
        ui.horizontal(|ui| {
            ui.label("size:");
            if ui.add(egui::DragValue::new(&mut config.size)).changed() {
                if let Some(orb) = meshes.get_mut(&**orb) {
                    *orb = Mesh::from(Circle::new(config.size))
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("pan:");
            ui.add(egui::DragValue::new(&mut config.pan));
        });
        ui.horizontal(|ui| {
            ui.label("zoom:");
            ui.add(egui::DragValue::new(&mut config.zoom));
        });
    });
    // on wasm this is shown on the webpage
    #[cfg(not(target_family = "wasm"))]
    egui::Window::new("about").show(ctx, |ui| {
        ui.label("skyweb is a fun thing i made because i randomly went down the atproto webtoy rabbit hole and wanted to make one!");
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.x = ui.fonts(|fonts| fonts.glyph_width(&egui::TextStyle::Body.resolve(ui.style()), ' '));
            ui.label("it was made possible by:");
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
    });
}
