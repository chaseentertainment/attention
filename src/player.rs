use crate::track::Track;
use discord_rich_presence::{
    DiscordIpc, DiscordIpcClient,
    activity::{Activity, ActivityType, Button},
};
use rodio::{Decoder, OutputStream, Sink};
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
    time::Duration,
};

pub struct Player {
    _stream_handle: OutputStream,
    sink: Sink,
    pub queue: Vec<Track>,
    // index from the array of tracks, needed to skip and such
    pub queue_index: usize,
    // playhead holds current timestamp in the track
    pub playhead: u64,
    pub volume: f32,
    pub discord: Option<DiscordIpcClient>,
}

impl Player {
    pub fn new(discord_presence: bool) -> Self {
        let _stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");

        let sink = rodio::Sink::connect_new(_stream_handle.mixer());

        let discord = if let Ok(mut client) = DiscordIpcClient::new("1396555007951638770") {
            match client.connect() {
                Ok(c) => Some(c),
                Err(_) => None,
            };

            Some(client)
        } else {
            None
        };

        Self {
            _stream_handle,
            sink,
            queue: vec![],
            queue_index: 0,
            playhead: 0,
            volume: 1.0,
            discord: if discord_presence { discord } else { None },
        }
    }

    pub fn update(&mut self) {
        self.playhead = self.sink.get_pos().as_secs();

        if self.sink.volume() != self.volume {
            self.sink.set_volume(self.volume);
        }

        if self.sink.empty() && self.can_next() {
            self.next();
        }
    }

    pub fn playing(&self) -> bool {
        !self.sink.is_paused() && !self.sink.empty()
    }

    pub fn play(&mut self) {
        self.sink.play();
    }

    pub fn pause(&mut self) {
        self.sink.pause();
    }

    pub fn can_next(&self) -> bool {
        self.queue_index + 1 != self.queue.len()
    }

    pub fn can_prev(&self) -> bool {
        self.queue_index != 0
    }

    pub fn next(&mut self) {
        self.play_track(self.queue_index + 1);
    }

    pub fn prev(&mut self) {
        self.play_track(self.queue_index - 1);
    }

    pub fn update_playhead(&mut self) {
        self.sink.try_seek(Duration::from_secs(self.playhead)).ok();
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, value: f32) {
        self.volume = value
    }

    pub fn track(&self) -> Option<&Track> {
        self.queue.get(self.queue_index)
    }

    pub fn load_library_into_queue(&mut self, path: &PathBuf) {
        self.queue.clear();

        let contents = match fs::read_dir(path) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("can't read from library path: {e}");
                return;
            }
        };

        for path in contents.flatten() {
            let track = match Track::new(path.path()) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("unable to load track: {e}");
                    continue;
                }
            };

            self.queue.push(track);
        }
    }

    pub fn play_track(&mut self, index: usize) {
        self.sink.clear();

        let track = match self.queue.get(index) {
            Some(t) => t,
            None => {
                eprintln!("invalid queue index");
                return;
            }
        };

        let source = match Self::get_audio_source(track) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("unable to retrieve audio source: {e}");
                return;
            }
        };

        self.sink.append(source);
        self.sink.play();
        self.queue_index = index;

        if let Some(client) = &mut self.discord {
            if let Err(e) = client.set_activity(
                Activity::new()
                    .details(&track.artist)
                    .state(&track.title)
                    .buttons(vec![Button::new(
                        "attention music player",
                        "https://github.com/chasetripleseven/attention",
                    )])
                    .activity_type(ActivityType::Listening),
            ) {
                eprintln!("unable to set discord presence: {e}");
            }
        }
    }

    fn get_audio_source(track: &Track) -> anyhow::Result<Decoder<BufReader<File>>> {
        let file = File::open(&track.path)?;
        Ok(Decoder::try_from(file)?)
    }
}
