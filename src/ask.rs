use super::*;
use atrium_api::app::bsky::actor::get_profile;

pub struct Stuff;

impl Plugin for Stuff {
    fn build(&self, app: &mut App) {
        app.init_resource::<Ask>()
            .add_systems(
                bevy_egui::EguiPrimaryContextPass,
                ask.run_if(in_state(Game::Ask)),
            )
            .add_systems(Update, check.run_if(in_state(Game::Ask)));
    }
}

fn ask(mut ctx: bevy_egui::EguiContexts, mut ask: ResMut<Ask>) {
    use bevy_egui::egui;
    let Ok(ctx) = ctx.ctx_mut() else { return };
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered_justified(|ui| {
            let size = &mut ui
                .style_mut()
                .text_styles
                .get_mut(&egui::TextStyle::Body)
                .unwrap()
                .size;
            *size *= 4.0;
            let size = *size;
            let width = &mut ui.spacing_mut().text_edit_width;
            *width *= 2.0;
            let width = *width;
            ui.allocate_space(egui::Vec2::new(0.0, (ui.available_height() - size) / 2.0));
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - width) / 2.0);
                if (ui
                    .add(
                        egui::TextEdit::singleline(&mut ask.buf)
                            .hint_text("enter your bsky handle"),
                    )
                    .lost_focus()
                    && ui.input(|input| input.key_pressed(egui::Key::Enter))
                    || ui.button("go").clicked())
                    && let Ok(actor) = ask
                        .buf
                        .parse::<atrium_api::types::string::AtIdentifier>()
                        .or_else(|_| (ask.buf.clone() + ".bsky.social").parse())
                {
                    ask.task = Some(
                        bevy::tasks::IoTaskPool::get().spawn(Compat::new(
                            CLIENT.service.app.bsky.actor.get_profile(
                                get_profile::ParametersData {
                                    actor: actor.clone(),
                                }
                                .into(),
                            ),
                        )),
                    );
                }
            });
            if let Some(err) = ask.err.as_ref() {
                ui.colored_label(egui::Color32::RED, err);
            }
        })
    });
}

#[derive(Resource, Default)]
struct Ask {
    buf: String,
    err: Option<String>,
    task: Option<
        bevy::tasks::Task<atrium_api::xrpc::Result<get_profile::Output, get_profile::Error>>,
    >,
}

fn check(mut commands: Commands, mut ask: ResMut<Ask>, mut next: ResMut<NextState<Game>>) {
    match ask
        .task
        .as_mut()
        .and_then(|task| bevy::tasks::block_on(bevy::tasks::futures_lite::future::poll_once(task)))
    {
        Some(Ok(profile)) => {
            commands.insert_resource(Profile {
                actor: profile.handle.parse().unwrap(),
                profile: profile.data,
            });
            ask.buf.clear();
            next.set(Game::Get)
        }
        Some(Err(e)) => ask.err = Some(e.to_string()),
        None => return,
    }
    ask.task = None;
}
