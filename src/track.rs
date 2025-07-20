use anyhow::Error;
use lofty::{
    file::{AudioFile, TaggedFileExt},
    tag::Accessor,
};
use std::{borrow::Cow, path::PathBuf, time::Duration};

#[derive(Clone, Debug)]
pub struct Track {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub duration: Duration,
    pub _bitrate: Option<u32>,
}

impl Track {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        let track: Self;

        let data = match lofty::read_from_path(&path) {
            Ok(d) => d,
            Err(e) => {
                return Err(Error::msg(format!("can't read tagged audio file: {e}")));
            }
        };

        let duration = data.properties().duration();
        let bitrate = data.properties().overall_bitrate();

        let tag = match data.primary_tag() {
            Some(t) => t,
            None => {
                track = Self {
                    path: path.clone(),
                    title: String::from(path.to_string_lossy()),
                    artist: String::from("unknown"),
                    duration,
                    _bitrate: bitrate,
                };

                return Ok(track);
            }
        };

        let file_name = match path.file_name() {
            Some(n) => n.to_string_lossy(),
            None => Cow::from("unknown path"),
        };

        let title = tag.title().unwrap_or(file_name).to_string();
        let artist = tag.artist().unwrap_or(Cow::from("unknown")).to_string();

        track = Self {
            path,
            title,
            artist,
            duration,
            _bitrate: bitrate,
        };

        Ok(track)
    }
}
