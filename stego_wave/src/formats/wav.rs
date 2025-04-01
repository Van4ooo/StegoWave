use crate::configuration::Settings;
use crate::error::StegoError;
use crate::object::{
    AudioFileSpec, AudioSteganography, ByteIterator, ResultStego, UniqueRandomIndices,
};
use derive_builder::Builder;
use std::io::Cursor;
use std::iter;
use std::path::{Path, PathBuf};

/// A steganography encoder/decoder for 16-bit WAV audio files.
///
/// This struct provides methods for hiding and extracting messages in 16-bit WAV files.
///
/// # Examples
///
/// ```rust
/// # use stego_wave::{formats::wav::WAV16, AudioSteganography, configuration::Settings};
/// let wav16 = WAV16::builder().lsb_deep(1).settings(Settings::new("../sw_config.toml").unwrap()).build().unwrap();
/// ```
#[derive(Builder, Debug, PartialEq)]
#[builder(build_fn(validate = "Self::validate"))]
#[builder(name = "WAV16Builder")]
pub struct WAV16 {
    lsb_deep: u8,
    #[builder(default)]
    settings: Settings,
}

impl Default for WAV16 {
    fn default() -> Self {
        let setting = Settings::default();
        WAV16::builder()
            .lsb_deep(setting.stego_wave_lib.default_lsb_deep)
            .settings(setting)
            .build()
            .unwrap()
    }
}

impl WAV16Builder {
    fn validate(&self) -> Result<(), String> {
        if let Some(ref lsb_deep) = self.lsb_deep {
            match *lsb_deep {
                ld if ld == 0 || ld > 16 => Err("lsb_deep must be between 1 and 16".to_string()),
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }
}

impl WAV16 {
    fn max_occupancy(&self) -> usize {
        self.settings.stego_wave_lib.max_occupancy
    }

    fn header(&self) -> &str {
        &self.settings.stego_wave_lib.header
    }

    fn is_enough_samples(&self, msg_len: usize, samples_len: usize) -> ResultStego<()> {
        let msg_bits = msg_len * 8;
        let total_bits = samples_len * (self.lsb_deep as usize) * self.max_occupancy() / 100;

        if msg_bits > total_bits {
            let required_bits = msg_bits * 100 / self.max_occupancy();
            let required_samples = required_bits / (self.lsb_deep as usize);
            return Err(StegoError::NotEnoughSamples(required_samples + 1));
        }

        Ok(())
    }

    fn read_sample(reader: &mut hound::WavReader<impl std::io::Read>) -> ResultStego<Vec<i16>> {
        reader
            .samples::<i16>()
            .map(|s| s.map_err(StegoError::from))
            .collect()
    }

    #[inline]
    fn get_mask(&self) -> i16 {
        let mask: i32 = (1 << self.lsb_deep) - 1;
        mask as i16
    }

    fn validate_header<'a, I: Iterator<Item = usize>>(
        &self,
        samples: &'a [i16],
        indicates_iter: &'a mut I,
    ) -> ResultStego<ByteIterator<'a, &'a mut I, i16>> {
        let mut header_bytes = Vec::with_capacity(self.header().len());

        let mut byte_iterator = ByteIterator::new(
            samples,
            indicates_iter,
            self.get_mask(),
            self.lsb_deep,
            0,
            0,
        );

        for byte in &mut byte_iterator {
            header_bytes.push(byte);

            if header_bytes.len() == self.header().len() {
                break;
            }
        }

        if header_bytes == self.header().as_bytes() {
            Ok(byte_iterator)
        } else {
            Err(StegoError::IncorrectPassword)
        }
    }
}

impl AudioSteganography<i16> for WAV16 {
    type Builder = WAV16Builder;

    fn builder() -> Self::Builder {
        WAV16Builder::default()
    }

    /// Hides a secret message in a 16-bit WAV file.
    ///
    /// Reads the input file, hides the message, and writes the output file.
    ///
    /// # Arguments
    ///
    /// * `file_input` - Path to the input WAV file.
    /// * `file_output` - Path where the output WAV file will be saved.
    /// * `message` - The message to hide.
    /// * `password` - The password used for steganography.
    ///
    /// # Note
    ///
    /// See test_hide_and_extract_message() test for usage example.
    fn hide_message(
        &self,
        file_input: impl Into<PathBuf>,
        file_output: impl Into<PathBuf>,
        message: impl Into<String>,
        password: impl Into<String>,
    ) -> ResultStego<()> {
        let input_path = file_input.into();
        let output_path = file_output.into();

        self.validate_file(&input_path)?;
        let mut reader = hound::WavReader::open(&input_path)?;
        let mut samples = Self::read_sample(&mut reader)?;

        self.hide_message_binary(&mut samples, &message.into(), &password.into())?;

        let mut writer = hound::WavWriter::create(output_path, reader.spec())?;
        for sample in samples {
            writer.write_sample(sample)?;
        }
        writer.finalize()?;
        Ok(())
    }

    /// Hides a secret message in an array of samples.
    ///
    /// This function embeds the message into the provided samples using the specified password.
    ///
    /// # Arguments
    ///
    /// * `samples` - A mutable slice of audio samples.
    /// * `message` - The message to hide.
    /// * `password` - The password used for random index generation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use stego_wave::{formats::wav::WAV16, AudioSteganography, configuration::Settings};
    /// # let wav16 = WAV16::builder().lsb_deep(1).settings(Settings::new("../sw_config.toml").unwrap()).build().unwrap();
    ///
    /// let mut samples = vec![8; 1_000];
    /// wav16.hide_message_binary(&mut samples, "Test message", "_").unwrap();
    /// let res = wav16.extract_message_binary(&samples, "_").unwrap();
    /// assert_eq!(res, "Test message");
    /// ```
    fn hide_message_binary(
        &self,
        samples: &mut [i16],
        message: &str,
        password: &str,
    ) -> ResultStego<()> {
        let header_bytes = self.header().as_bytes();
        let message_bytes = message.as_bytes();

        let total_bytes = header_bytes.len() + message_bytes.len() + 1;
        self.is_enough_samples(total_bytes, samples.len())?;

        let mask = !self.get_mask();
        let indices_iter = UniqueRandomIndices::new(samples.len(), password, self.max_occupancy());
        let mut message = header_bytes
            .iter()
            .chain(message_bytes.iter())
            .chain(iter::once(&0))
            .flat_map(|&byte| (0..8).rev().map(move |shift| (byte >> shift) & 1));

        let mut write_full = false;
        'sample: for sample_index in indices_iter {
            let mut value: u16 = 0;
            for _ in 0..self.lsb_deep {
                value = (value << 1)
                    | (if let Some(bit) = message.next() {
                        bit as u16
                    } else {
                        write_full = true;
                        0u16
                    });
            }

            samples[sample_index] = (samples[sample_index] & mask) | (value as i16);
            if write_full {
                break 'sample;
            }
        }
        Ok(())
    }

    /// Extracts a hidden secret message from a 16-bit WAV file.
    ///
    /// # Arguments
    ///
    /// * `file` - Path to the WAV file containing the hidden message.
    /// * `password` - The password used during embedding.
    ///
    /// # Note
    ///
    /// See test_hide_and_extract_message() test for usage example.
    fn extract_message(
        &self,
        file: impl Into<PathBuf>,
        password: impl Into<String>,
    ) -> ResultStego<String> {
        let input_path = file.into();
        self.validate_file(&input_path)?;

        let mut reader = hound::WavReader::open(&input_path)?;
        let samples = Self::read_sample(&mut reader)?;

        self.extract_message_binary(&samples, &password.into())
    }

    /// Extracts a hidden secret message from an array of samples.
    ///
    /// This function retrieves the message embedded in the samples using the provided password.
    ///
    /// # Arguments
    ///
    /// * `samples` - A slice of audio samples.
    /// * `password` - The password used for embedding.
    ///
    /// # Errors
    ///
    /// Returns an error if extraction fails or if the password is incorrect.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use stego_wave::{formats::wav::WAV16, AudioSteganography, configuration::Settings};
    /// # let wav16 = WAV16::builder().lsb_deep(1).settings(Settings::new("../sw_config.toml").unwrap()).build().unwrap();
    ///
    /// let mut samples = vec![8; 1_000];
    /// wav16.hide_message_binary(&mut samples, "Test message", "_").unwrap();
    /// let res = wav16.extract_message_binary(&samples, "_").unwrap();
    /// assert_eq!(res, "Test message");
    /// ```
    fn extract_message_binary(&self, samples: &[i16], password: &str) -> ResultStego<String> {
        let mut indices_iter =
            UniqueRandomIndices::new(samples.len(), password, self.max_occupancy());

        let byte_iter = self.validate_header(samples, &mut indices_iter)?;
        let mut result: Vec<u8> = Vec::new();

        for byte in byte_iter {
            if byte == 0 {
                return Ok(String::from_utf8(result).unwrap_or_default());
            }
            result.push(byte);
        }

        Err(StegoError::FailedToReceiveMessage)
    }

    /// Clears the secret message embedded in a WAV file using the given password.
    ///
    /// # Arguments
    /// * `file` - The path to the WAV file from which to clear the secret message.
    /// * `password` - The password used to generate the unique sequence of indices.
    ///
    /// # Returns
    /// * `Ok(())` if the message is successfully cleared.
    /// * `Err(StegoError)` if an error occurs during the process.
    ///
    /// # Examples
    /// ```
    /// # use stego_wave::{formats::wav::WAV16, AudioSteganography, configuration::Settings};
    /// # let wav16 = WAV16::builder().lsb_deep(1).settings(Settings::new("../sw_config.toml").unwrap()).build().unwrap();
    ///
    /// let _ = wav16.clear_secret_message("hidden_message.wav", "my_password");
    /// ```
    fn clear_secret_message(&self, file: impl Into<PathBuf>, password: &str) -> ResultStego<()> {
        let input_path = file.into();
        self.validate_file(&input_path)?;

        let mut reader = hound::WavReader::open(&input_path)?;
        let mut samples = Self::read_sample(&mut reader)?;

        self.clear_secret_message_binary(&mut samples, password)?;

        let mut writer = hound::WavWriter::create(&input_path, reader.spec())?;
        for sample in samples {
            writer.write_sample(sample)?;
        }
        writer.finalize()?;

        Ok(())
    }

    /// Clears the binary representation of the secret message from the given samples.
    ///
    /// # Arguments
    /// * `samples` - A mutable slice of audio samples.
    /// * `password` - The password used to generate unique sample indices.
    ///
    /// # Returns
    /// * `Ok(())` if the secret message is successfully cleared.
    /// * `Err(StegoError)` if the message cannot be cleared or extracted.
    ///
    /// # Examples
    /// ```
    ///
    /// # use stego_wave::{formats::wav::WAV16, AudioSteganography, configuration::Settings};
    /// # let wav16 = WAV16::builder().lsb_deep(1).settings(Settings::new("../sw_config.toml").unwrap()).build().unwrap();
    ///
    /// let mut samples = vec![1000, 2000, 3000, 4000];
    /// let _ = wav16.clear_secret_message_binary(&mut samples, "my_password");
    /// ```
    fn clear_secret_message_binary(&self, samples: &mut [i16], password: &str) -> ResultStego<()> {
        let indices_iter = UniqueRandomIndices::new(samples.len(), password, self.max_occupancy());
        let mask = self.get_mask();

        self.validate_header(samples, &mut indices_iter.clone())?;

        let (mut current_byte, mut bit_count) = (0_u8, 0_u8);
        for sample_index in indices_iter {
            let encoded = (samples[sample_index] & mask) as u16;
            samples[sample_index] &= !mask;

            for shift in (0..self.lsb_deep).rev() {
                let bit = ((encoded >> shift) & 1) as u8;
                current_byte = (current_byte << 1) | bit;
                bit_count += 1;

                if bit_count == 8 {
                    if current_byte == 0 {
                        return Ok(());
                    }

                    current_byte = 0;
                    bit_count = 0;
                }
            }
        }
        Err(StegoError::FailedToReceiveMessage)
    }

    /// Validates that the provided WAV file is valid.
    ///
    /// # Arguments
    ///
    /// * `file` - A reference to the path of the WAV file.
    ///
    /// # Errors
    ///
    /// Returns an error if the WAV file does not use 16 bits per sample.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use stego_wave::{formats::wav::WAV16, AudioSteganography};
    /// let wav = WAV16::default();
    /// // Assuming "audio.wav" is a valid 16-bit WAV file:
    /// wav.validate_file(std::path::Path::new("audio.wav")).unwrap();
    /// ```
    fn validate_file(&self, file: &Path) -> ResultStego<()> {
        let reader = hound::WavReader::open(file)?;
        if reader.spec().bits_per_sample != 16 {
            return Err(StegoError::InvalidFile(
                "Only 16-bit WAV file supported".to_string(),
            ));
        }
        Ok(())
    }

    fn read_samples_from_byte(&self, byte: Vec<u8>) -> ResultStego<(Vec<i16>, AudioFileSpec)> {
        let cursor = Cursor::new(byte);
        let mut reader = hound::WavReader::new(cursor)
            .map_err(|_| StegoError::Other("Error reading WAV".to_string()))?;

        let spec = reader.spec();
        let samples = WAV16::read_sample(&mut reader)
            .map_err(|_| StegoError::Other("Error reading samples".to_string()))?;

        Ok((samples, AudioFileSpec::Wav(spec)))
    }

    fn write_samples_to_byte(&self, spec: AudioFileSpec, samples: &[i16]) -> ResultStego<Vec<u8>> {
        let mut out_buf = Cursor::new(Vec::<u8>::new());
        let mut writer = match spec {
            AudioFileSpec::Wav(spec) => hound::WavWriter::new(&mut out_buf, spec)?,
        };

        for sample in samples {
            writer
                .write_sample(*sample)
                .map_err(|_| StegoError::Other("Error writing sample".to_string()))?;
        }

        writer
            .finalize()
            .map_err(|_| StegoError::Other("Error finalizing writer".to_string()))?;

        Ok(out_buf.into_inner())
    }

    fn default_filename(&self) -> String {
        "wav_16.wav".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::error::Error;
    use std::fs;

    const CONFIG_FILE: &str = "../sw_config.toml";

    #[test]
    fn extract_message_binary_success() -> Result<(), Box<dyn Error>> {
        let sample: &mut [i16; 1_000] = &mut [8; 1_000];
        for i in 1..17 {
            let wav16 = WAV16::builder()
                .lsb_deep(i)
                .settings(Settings::new(CONFIG_FILE).unwrap())
                .build()?;

            wav16.hide_message_binary(sample, &format!("{i} test {i}"), "_")?;
            let res = wav16.extract_message_binary(sample, "_")?;
            assert_eq!(res, format!("{i} test {i}"));
        }

        Ok(())
    }

    #[test]
    fn extract_message_binary_failed() -> Result<(), Box<dyn Error>> {
        let sample: &mut [i16; 1_000] = &mut [8; 1_000];
        let wav16 = WAV16::builder()
            .lsb_deep(1)
            .settings(Settings::new(CONFIG_FILE).unwrap())
            .build()?;

        wav16.hide_message_binary(sample, "test", "qwerty1")?;

        assert!(wav16.extract_message_binary(sample, "qwerty2").is_err());
        assert!(wav16.extract_message_binary(sample, "qwerty").is_err());
        assert!(wav16.extract_message_binary(sample, "qwerty1").is_ok());
        assert!(wav16.extract_message_binary(sample, "qwerty1").is_ok());

        Ok(())
    }

    fn create_wav_file(
        path: &PathBuf,
        bits_per_sample: u16,
        samples: &[i16],
    ) -> Result<(), Box<dyn Error>> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 44_100,
            bits_per_sample,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(path, spec)?;
        for &sample in samples {
            writer.write_sample(sample)?
        }

        writer.finalize()?;
        Ok(())
    }

    fn temp_path(filename: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(filename);
        path
    }

    #[test]
    fn test_hide_and_extract_message() -> Result<(), Box<dyn Error>> {
        let samples: Vec<i16> = vec![0; 10_000];
        let input_path = temp_path("input_full.wav");
        let output_path = temp_path("output_full.wav");
        create_wav_file(&input_path, 16, &samples)?;

        let wav16 = WAV16::builder()
            .lsb_deep(1)
            .settings(Settings::new(CONFIG_FILE).unwrap())
            .build()?;

        let message = "Hello World!";
        let password = "qwerty1234";

        wav16.hide_message(&input_path, &output_path, message, password)?;
        let res = wav16.extract_message(&output_path, password)?;

        assert_eq!(res, message);

        let _ = fs::remove_file(input_path);
        let _ = fs::remove_file(output_path);

        Ok(())
    }

    #[test]
    fn test_incorrect_password() -> Result<(), Box<dyn Error>> {
        let samples: Vec<i16> = vec![0; 10_000];

        let input_path = temp_path("input_incorrect.wav");
        let output_path = temp_path("output_incorrect.wav");
        create_wav_file(&input_path, 16, &samples)?;

        let wav16 = WAV16::builder()
            .lsb_deep(1)
            .settings(Settings::new(CONFIG_FILE).unwrap())
            .build()?;

        let message = "Hello World!";
        let password = "qwerty1234";

        wav16.hide_message(&input_path, &output_path, message, password)?;
        let res = wav16.extract_message(&output_path, "wrong_password");

        match res {
            Err(StegoError::IncorrectPassword) => (),
            _ => assert!(false),
        }

        let _ = fs::remove_file(input_path);
        let _ = fs::remove_file(output_path);

        Ok(())
    }

    #[test]
    fn test_clear_secret_message() -> Result<(), Box<dyn Error>> {
        let samples: Vec<i16> = vec![0; 10_000];
        let input_path = temp_path("input_clear.wav");
        let output_path = temp_path("output_clear.wav");
        create_wav_file(&input_path, 16, &samples)?;

        let wav16 = WAV16::builder()
            .lsb_deep(1)
            .settings(Settings::new(CONFIG_FILE).unwrap())
            .build()?;

        let message = "Hello World!";
        let password = "qwerty1234";

        wav16.hide_message(&input_path, &output_path, message, password)?;
        let res = wav16.extract_message(&output_path, password)?;
        assert_eq!(res, message);

        wav16.clear_secret_message(&output_path, password)?;

        match wav16.extract_message(&output_path, password) {
            Err(StegoError::IncorrectPassword) => assert!(true),
            _ => assert!(false),
        }

        let _ = fs::remove_file(input_path);
        let _ = fs::remove_file(output_path);

        Ok(())
    }

    #[test]
    fn test_incorrect_bits_per_sample() -> Result<(), Box<dyn Error>> {
        let samples: Vec<i16> = vec![0; 10_000];
        let input_path = temp_path("input_incorrect_bits_per_sample.wav");
        let output_path = temp_path("output_incorrect_bits_per_sample.wav");
        create_wav_file(&input_path, 8, &samples)?;

        let res = WAV16::builder()
            .lsb_deep(1)
            .settings(Settings::new(CONFIG_FILE).unwrap())
            .build()?
            .hide_message(&input_path, &output_path, "test", "rest");

        match res {
            Err(StegoError::InvalidFile(err)) => assert_eq!(err, "Only 16-bit WAV file supported"),
            _ => assert!(false),
        }

        let _ = fs::remove_file(input_path);
        let _ = fs::remove_file(output_path);

        Ok(())
    }
}
