use super::*;

pub struct Ask;

impl Plugin for Ask {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_simple_text_input::TextInputPlugin)
            .add_systems(OnEnter(Game::Ask), spawn)
            .add_systems(Update, submit.run_if(in_state(Game::Ask)))
            .add_observer(report);
    }
}

#[derive(Component)]
struct Report;

fn spawn(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(Camera2d);
    let font = assets.load("grapesoda.ttf");
    commands.spawn((
        StateScoped(Game::Ask),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            (
                Text::new("enter your bsky handle:"),
                TextFont {
                    font: font.clone_weak(),
                    font_size: 50.,
                    font_smoothing: bevy::text::FontSmoothing::None,
                    line_height: bevy::text::LineHeight::RelativeToFont(1.),
                },
                TextColor(bevy::color::palettes::css::DEEP_SKY_BLUE.into()),
            ),
            (
                bevy_simple_text_input::TextInput,
                bevy_simple_text_input::TextInputTextFont(TextFont {
                    font: font.clone_weak(),
                    font_size: 50.,
                    font_smoothing: bevy::text::FontSmoothing::None,
                    line_height: bevy::text::LineHeight::RelativeToFont(1.),
                }),
                bevy_simple_text_input::TextInputTextColor(TextColor(
                    bevy::color::palettes::css::AZURE.into()
                )),
            ),
            (
                Report,
                Text::new(""),
                TextFont {
                    font,
                    font_size: 50.,
                    font_smoothing: bevy::text::FontSmoothing::None,
                    line_height: bevy::text::LineHeight::RelativeToFont(1.),
                },
                TextColor(bevy::color::palettes::css::RED.into()),
            ),
        ],
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

#[derive(Event)]
struct Bad(String);

fn report(trigger: Trigger<Bad>, mut report: Single<&mut Text, With<Report>>) {
    report.0 = trigger.event().0.clone();
}

async fn check(mut ctx: bevy_tokio_tasks::TaskContext, handle: String) {
    let actor: atrium_api::types::string::AtIdentifier = match handle.parse() {
        Ok(actor) => actor,
        Err(_) => {
            ctx.run_on_main_thread(|bevy| bevy.world.trigger(Bad("invalid handle".into())))
                .await;
            return;
        }
    };
    let client = client();
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
        }
        Err(e) => {
            ctx.run_on_main_thread(move |bevy| bevy.world.trigger(Bad(e.to_string())))
                .await
        }
    }
}
