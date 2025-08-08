use super::*;

pub struct Ask;

impl Plugin for Ask {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Game::Ask), spawn)
            .add_systems(Update, buttons.run_if(in_state(Game::Ask)));
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
        children![
            (
                Text("skyweb".into()),
                TextFont {
                    font: assets.load("grapesoda.ttf"),
                    font_size: 50.,
                    font_smoothing: bevy::text::FontSmoothing::None,
                    line_height: bevy::text::LineHeight::RelativeToFont(1.),
                },
                TextColor(bevy::color::palettes::css::AZURE.into()),
            ),
            (
                Button,
                BackgroundColor(Color::NONE),
                children![(
                    Text("uwu".into()),
                    TextFont {
                        font: assets.load("grapesoda.ttf"),
                        font_size: 50.,
                        font_smoothing: bevy::text::FontSmoothing::None,
                        line_height: bevy::text::LineHeight::RelativeToFont(1.),
                    },
                    TextColor(bevy::color::palettes::css::GOLD.into()),
                )]
            )
        ],
    ));
}

fn buttons(
    mut buttons: Query<(&Interaction, &Children), (With<Button>, Changed<Interaction>)>,
    mut text: Query<&mut Text>,
    mut state: ResMut<NextState<Game>>,
) {
    let Ok((interaction, children)) = buttons.single_mut() else {
        return;
    };
    text.get_mut(children[0]).unwrap().0 = match interaction {
        Interaction::Pressed => ">w<",
        Interaction::Hovered => "owo",
        Interaction::None => "uwu",
    }
    .into();
    if let Interaction::Pressed = interaction {
        state.set(Game::Login)
    }
}
