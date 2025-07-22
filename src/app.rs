use crate::{
    config::{Config, load_config},
    player::Player,
};
use discord_rich_presence::DiscordIpc;
use egui::{Checkbox, Label, RichText, Sense, Slider};

pub struct Attention {
    config: Config,
    player: Player,
}

impl Default for Attention {
    fn default() -> Self {
        let config = match load_config() {
            Ok(c) => c,
            Err(_) => Config {
                library_path: None,
                discord_presence: true,
            },
        };

        let mut player = Player::new(config.discord_presence);

        if let Some(ref library) = config.library_path {
            player.load_library_into_queue(&library);
        }

        if player.queue.len() > 0 {
            player.play_track(player.queue_index);
            player.pause();
        }

        Self { config, player }
    }
}

impl eframe::App for Attention {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(client) = &mut self.player.discord {
            client.close().ok();
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.player.update();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("attention");

            ui.horizontal(|ui| {
                let add_library_label = if self.config.library_path.is_none() {
                    "select library"
                } else {
                    "change library"
                };

                if ui.button(add_library_label).clicked() {
                    if let Some(library) = self.config.set_library() {
                        self.player.load_library_into_queue(&library);
                    }
                }

                if let Some(path) = &self.config.library_path {
                    ui.label(path.to_string_lossy());
                }
            });

            if self.config.library_path.is_some() {
                ui.separator();

                ui.horizontal(|ui| {
                    if self.player.playing() {
                        if ui.button("pause").clicked() {
                            self.player.pause();
                        }
                    } else if ui.button("play").clicked() {
                        self.player.play();
                    }

                    if self.player.can_prev() && ui.button("prev").clicked() {
                        self.player.prev();
                    }

                    if self.player.can_next() && ui.button("next").clicked() {
                        self.player.next();
                    }

                    if let Some(track) = self.player.track().cloned() {
                        ui.add(Label::new(RichText::new(track.artist).size(16.0)));
                        ui.add(Label::new(track.title));

                        let duration_slider = ui.add(Slider::new(
                            &mut self.player.playhead,
                            0..=track.duration.as_secs(),
                        ));

                        if duration_slider.dragged() {
                            self.player.update_playhead();
                        }
                    }
                });

                ui.add(Slider::new(&mut self.player.volume, 0.0..=1.0).text("volume"));
                if self.player.volume != self.player.volume() {
                    self.player.set_volume(self.player.volume);
                }

                ui.horizontal(|ui| {
                    if ui
                        .add(Checkbox::new(
                            &mut self.config.discord_presence,
                            "discord presence (restart to apply)",
                        ))
                        .changed()
                    {
                        self.config
                            .set_discord_presence(self.config.discord_presence);
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, track) in self.player.queue.clone().iter().enumerate() {
                        let file_name = match track.path.file_name() {
                            Some(n) => n.to_string_lossy(),
                            None => {
                                eprintln!("unable to determine file name");
                                continue;
                            }
                        };

                        let is_current_track = match self.player.track() {
                            Some(t) => t.path == track.path,
                            None => false,
                        };

                        let label = Label::new(file_name).sense(Sense::click());

                        let label_widget = if is_current_track {
                            ui.add(label).highlight()
                        } else {
                            ui.add(label)
                        };

                        if label_widget.clicked() {
                            self.player.play_track(i);
                        }
                    }
                });
            }
        });
    }
}
