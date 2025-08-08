use super::*;

pub struct Ask;

impl Plugin for Ask {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_simple_text_input::TextInputPlugin)
            .add_systems(OnEnter(Game::Ask), spawn)
            .add_systems(Update, submit.run_if(in_state(Game::Ask)));
    }
}

fn spawn(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn((
        StateScoped(Game::Ask),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            bevy_simple_text_input::TextInput,
            bevy_simple_text_input::TextInputTextFont(TextFont {
                font: assets.load("grapesoda.ttf"),
                font_size: 50.,
                font_smoothing: bevy::text::FontSmoothing::None,
                line_height: bevy::text::LineHeight::RelativeToFont(1.),
            }),
            bevy_simple_text_input::TextInputTextColor(TextColor(
                bevy::color::palettes::css::AZURE.into()
            )),
        )],
    ));
}

fn submit(
    mut events: EventReader<bevy_simple_text_input::TextInputSubmitEvent>,
    mut next: ResMut<NextState<Game>>,
) {
    for event in events.read() {
        next.set(Game::Login)
    }
}
