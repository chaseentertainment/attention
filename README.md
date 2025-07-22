# attention
minimal graphical music player

<img src="screenshots/image.png" width="600">

# known issues
- some applications might refuse from working and complain about attention claiming audio streams exclusively for itself
- i've observed that the player does not skip to the next song automatically when it's not in focus, but can't reproduce it reliably

# features
- select a library of audio files
- automatically play next song or skip manually
- adjust the volume
- start over when the queue is done
- pause and resume playback
- skip and rewind the audio track
- display title and artist tags
- discord presence that shows title and artist name

# build instructions
- have cargo and rustc installed
- run `cargo build --release`
