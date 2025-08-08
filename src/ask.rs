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
    tokio: Res<bevy_tokio_tasks::TokioTasksRuntime>,
) {
    for event in events.read() {
        let handle = event.value.clone();
        tokio.spawn_background_task(|ctx| check(ctx, handle));
    }
}

async fn check(mut ctx: bevy_tokio_tasks::TaskContext, handle: String) {
    let actor: atrium_api::types::string::AtIdentifier = match handle.parse() {
        Ok(actor) => actor,
        Err(_) => todo!(),
    };
    let client = client();
    loop {
        match client
            .service
            .app
            .bsky
            .actor
            .get_profile(
                atrium_api::app::bsky::actor::get_profile::ParametersData {
                    actor: actor.clone(),
                }
                .into(),
            )
            .await
        {
            Ok(profile) => {
                ctx.run_on_main_thread(|bevy| {
                    bevy.world.insert_resource(Profile {
                        actor,
                        profile: profile.data,
                    });
                    bevy.world.resource_mut::<NextState<Game>>().set(Game::Get)
                })
                .await;
                break;
            }
            Err(_) => todo!(),
        }
    }
}
