use super::*;

pub struct Stuff;

impl Plugin for Stuff {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_egui::EguiPlugin::default())
            .add_systems(
                bevy_egui::EguiPrimaryContextPass,
                config.run_if(in_state(Game::Connect)),
            );
        // .add_systems(OnEnter(Game::Get), spawn);
    }
}

#[rustfmt::skip]
fn config(mut ctx: bevy_egui::EguiContexts, mut config: ResMut<Config>) -> Result {
    use bevy_egui::egui;
    egui::Window::new("").show(ctx.ctx_mut()?, |ui| {
        ui.heading("about");
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
                ui.hyperlink_to("bevy", "bevy.org");
                ui.label("which handles all the windowing, asset loading, state, ecs and start ui")
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
        ui.heading("config");
        ui.label("i've mostly been testing with my own account but the values may not suit your account so you can adjust those here");
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
    Ok(())
}

// i'll wait for bevy 0.17 to do config ui natively
/*
#[derive(Component)]
struct Toggle;

fn spawn(mut commands: Commands, font: Res<Grape>) {
    commands.spawn((
        (Toggle, Text::new("config")),
        (
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            Visibility::Hidden,
            BackgroundColor(bevy::color::palettes::css::DARK_SLATE_GRAY.into()),
            children![(
                Node {
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                children![(
                    Text::new("attraction:"),
                    TextFont {
                        font: font.clone_weak(),
                        font_smoothing: bevy::text::FontSmoothing::None,
                        ..default()
                    },
                    TextColor(bevy::color::palettes::css::AZURE.into()),
                ),],
            ),],
        ),
    ));
}
*/
