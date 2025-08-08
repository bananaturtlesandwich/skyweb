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
fn config(mut ctx: bevy_egui::EguiContexts, mut config: ResMut<Config>) {
    use bevy_egui::egui;
    let Ok(ctx) = ctx.ctx_mut() else { return };
    egui::Window::new("config").show(ctx,|ui| {
        ui.label("the default physics values may not suit your account so you can adjust those here");
        ui.horizontal(|ui| {
            ui.label("attraction:");
            ui.add(egui::DragValue::new(&mut config.attraction))
        });
        ui.horizontal(|ui| {
            ui.label("repulsion:");
            ui.add(egui::DragValue::new(&mut config.repulsion))
        });
        ui.horizontal(|ui| {
            ui.label("gravity:");
            ui.add(egui::DragValue::new(&mut config.gravity))
        });
        ui.horizontal(|ui| {
            ui.label("tick:");
            ui.add(egui::DragValue::new(&mut config.tick))
        });
    });
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
                ui.hyperlink_to("avian", "https://crates.io/crates/avian2d");
                ui.label("which resolves all the collisions really efficiently via the bevy ecs")
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("•");
                ui.hyperlink_to("bevy web asset", "https://crates.io/crates/bevy_web_asset");
                ui.label("which asynchronously grabs user avatars (i couldn't have done that)") 
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("•");
                ui.hyperlink_to("bevy tokio tasks", "https://crates.io/crates/bevy-tokio-tasks");
                ui.label("which provided the perfect interface to make atproto requests asynchronous from the game thread")
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("•");
                ui.hyperlink_to("bevy egui", "https://crates.io/crates/bevy_egui");
                ui.label("which was able to fill in for bevy while its widgets are still cooking")
            });
        });
    });
}
