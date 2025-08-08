use super::*;

pub struct Stuff;

impl Plugin for Stuff {
    fn build(&self, app: &mut App) {
        app.init_resource::<Ask>().add_systems(
            bevy_egui::EguiPrimaryContextPass,
            ask.chain().run_if(in_state(Game::Ask)),
        );
    }
}

fn ask(
    mut ctx: bevy_egui::EguiContexts,
    tokio: Res<bevy_tokio_tasks::TokioTasksRuntime>,
    mut ask: ResMut<Ask>,
) {
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
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut ask.buf)
                            .hint_text("enter your bsky handle"),
                    )
                    .lost_focus()
                    && ui.input(|input| input.key_pressed(egui::Key::Enter))
                    || ui.button("go").clicked()
                {
                    let buf = ask.buf.clone();
                    tokio.spawn_background_task(|ctx| check(ctx, buf));
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
}

async fn check(mut ctx: bevy_tokio_tasks::TaskContext, handle: String) {
    let actor: atrium_api::types::string::AtIdentifier = match handle
        .parse()
        .or_else(|_| (handle + ".bsky.social").parse())
    {
        Ok(actor) => actor,
        Err(_) => {
            ctx.run_on_main_thread(|bevy| {
                bevy.world.resource_mut::<Ask>().err = Some("invalid handle".into())
            })
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
            ctx.run_on_main_thread(move |bevy| {
                bevy.world.resource_mut::<Ask>().err = Some(e.to_string())
            })
            .await
        }
    }
}
