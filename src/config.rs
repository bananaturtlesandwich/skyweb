use super::*;

pub struct Stuff;

impl Plugin for Stuff {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Game::Get), spawn);
    }
}

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
            // should i just wait for bevy 0.17 for sliders?
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
