use crate::track::Track;
use egui::{Label, RichText, Sense, Slider};
use rfd::FileDialog;
use rodio::Decoder;
use std::{fs, io::Write, path::PathBuf, time::Duration};

pub struct Attention {
    library_path: Option<PathBuf>,
    current_track: Option<Track>,
    _stream_handle: rodio::OutputStream,
    sink: rodio::Sink,
    queue: Vec<PathBuf>,
    queue_index: usize,
    playing: bool,
    cursor: u64,
    volume: f32,
    config_file_path: PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Config {
    library_path: PathBuf,
}

impl Default for Attention {
    fn default() -> Self {
        let stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");

        let sink = rodio::Sink::connect_new(stream_handle.mixer());

        let mut attention = Self {
            library_path: None,
            current_track: None,
            _stream_handle: stream_handle,
            sink,
            queue: vec![],
            queue_index: 0,
            playing: false,
            cursor: 0,
            volume: 1.0,
            config_file_path: dirs::home_dir()
                .unwrap()
                .join(".config/attention/attention.json"),
        };

        attention.load_library_path();

        attention
    }
}

impl eframe::App for Attention {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            self.cursor = self.sink.get_pos().as_secs();

            ui.heading("attention");

            ui.horizontal(|ui| {
                let add_library_label = if self.library_path.is_none() {
                    "select library"
                } else {
                    "change library"
                };

                if ui.button(add_library_label).clicked() {
                    self.set_library_path();
                }

                if let Some(path) = &self.library_path {
                    ui.label(path.to_string_lossy());
                }
            });

            if self.library_path.is_some() {
                ui.separator();

                ui.horizontal(|ui| {
                    if self.playing {
                        if ui.button("pause").clicked() {
                            self.playing = false;
                            self.sink.pause();
                        }
                    } else if ui.button("play").clicked() {
                        self.playing = true;
                        if self.current_track.is_some() {
                            self.sink.play();
                        } else {
                            self.play_track(self.queue_index);
                        }
                    }

                    if let Some(track) = self.current_track.clone() {
                        if self.queue_index > 0 && ui.button("prev").clicked() {
                            self.play_track(self.queue_index - 1);
                        }

                        if self.queue_index + 1 < self.queue.len() - 1
                            && ui.button("next").clicked()
                        {
                            self.play_track(self.queue_index + 1);
                        }

                        ui.add(Label::new(RichText::new(track.artist).size(16.0)));
                        ui.add(Label::new(track.title));

                        let duration_slider =
                            ui.add(Slider::new(&mut self.cursor, 0..=track.duration.as_secs()));

                        if duration_slider.dragged() {
                            self.sink.try_seek(Duration::from_secs(self.cursor)).ok();
                        }
                    }
                });

                ui.add(Slider::new(&mut self.volume, 0.0..=1.0).text("Volume"));
                if self.volume != self.sink.volume() {
                    self.sink.set_volume(self.volume);
                }

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, path) in self.queue.clone().iter().enumerate() {
                        let file_name = match path.file_name() {
                            Some(n) => n.to_string_lossy(),
                            None => {
                                eprintln!("unable to determine file name");
                                continue;
                            }
                        };

                        let is_current_track = self.current_track.is_some()
                            && self.current_track.clone().unwrap().path == *path;

                        let label = Label::new(file_name).sense(Sense::click());

                        let label_widget = if is_current_track {
                            ui.add(label).highlight()
                        } else {
                            ui.add(label)
                        };

                        if label_widget.clicked() {
                            self.play_track(i);
                        }
                    }
                });

                if self.playing && self.sink.empty() {
                    if self.queue_index + 1 < self.queue.len() - 1 {
                        self.play_track(self.queue_index + 1);
                    } else {
                        self.play_track(0);
                    }
                }
            }
        });
    }
}

impl Attention {
    fn load_tracks_from_library_path(&mut self) {
        self.queue.clear();

        let path = match &self.library_path {
            Some(p) => p,
            None => {
                eprintln!("tried to load tracks from non-existing library");
                return;
            }
        };

        let contents = match fs::read_dir(path) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("can't read from library path: {e}");
                return;
            }
        };

        for path in contents.flatten() {
            self.queue.push(path.path());
        }
    }

    fn load_library_path(&mut self) {
        let serialized = match fs::read_to_string(self.config_file_path.clone()) {
            Ok(s) => s,
            Err(_) => {
                println!("config file not found, one will be created when choosing a library path");
                return;
            }
        };

        let config: Config = match serde_json::from_str(&serialized) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("invalid config file: {e}");
                return;
            }
        };

        self.library_path = Some(config.library_path);
        self.load_tracks_from_library_path();
    }

    fn set_library_path(&mut self) {
        self.library_path = FileDialog::new().pick_folder();
        self.load_tracks_from_library_path();

        if let Some(library_path) = &self.library_path {
            let config = Config {
                library_path: library_path.clone(),
            };

            let serialized = match serde_json::to_string_pretty(&config) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("unable to serialize config: {e}");
                    return;
                }
            };

            let config_dir = match self.config_file_path.parent() {
                Some(d) => d,
                None => {
                    eprintln!("unable to get config file directory");
                    return;
                }
            };

            if let Err(e) = fs::create_dir_all(config_dir) {
                eprintln!("unable to create config file directory: {e}");
            };

            let mut file = match fs::File::create(self.config_file_path.clone()) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("unable to create config file: {e}");
                    return;
                }
            };

            if let Err(e) = file.write_all(serialized.as_bytes()) {
                eprintln!("unable to write to config file: {e}");
            }
        }
    }

    fn play_track(&mut self, index: usize) {
        let path = match self.queue.get(index) {
            Some(p) => p,
            None => {
                eprintln!("invalid queue index");
                return;
            }
        };

        let track = match Track::new(path.clone()) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("unable to initialize track: {e}");
                return;
            }
        };

        self.current_track = Some(track);
        self.sink.clear();

        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("failed to play audio file: {e}");
                return;
            }
        };

        let source = match Decoder::try_from(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("failed to decode audio file: {e}");
                return;
            }
        };

        self.sink.append(source);
        self.sink.play();
        self.playing = true;
        self.queue_index = index;
    }
}
