use crate::AudioSteganography;
use crate::configuration::{Settings, StegoWaveLib};
use crate::formats::wav::WAV16;

pub mod wav;

pub fn get_stego_by_str(
    format: &str,
    lsb_deep: u8,
    settings: StegoWaveLib,
) -> Result<impl AudioSteganography<i16>, String> {
    match format {
        "wav16" => match WAV16::builder()
            .lsb_deep(lsb_deep)
            .settings(Settings {
                stego_wave_lib: settings,
            })
            .build()
        {
            Ok(wav16) => Ok(wav16),
            Err(err) => Err(format!("{err}")),
        },
        _ => Err("Invalid format".to_string()),
    }
}
