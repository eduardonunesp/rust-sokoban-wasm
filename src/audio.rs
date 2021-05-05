use ggez::{audio, Context};
use std::collections::HashMap;

#[derive(Default)]
pub struct AudioStore {
    pub sounds: HashMap<String, audio::SoundSource>,
}

impl AudioStore {
    pub fn play_sound(&mut self, sound: &String) {
        let _ = self
            .sounds
            .get_mut(sound)
            .expect("expected sound")
            .play_detached();
    }

    pub fn new(context: &mut Context) -> Self {
        let sounds_list = ["correct", "incorrect", "wall"];
        let mut sounds: HashMap<String, audio::SoundSource> = HashMap::new();

        for sound in sounds_list.iter() {
            let sound_name = sound.to_string();
            let sound_path = format!("/sounds/{}.wav", sound_name);
            let sound_source =
                audio::SoundSource::new(context, &sound_path).expect("expected sound loaded");

            sounds.insert(sound_name, sound_source);
        }

        AudioStore { sounds }
    }
}
