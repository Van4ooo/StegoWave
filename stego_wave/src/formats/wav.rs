use crate::error::StegoError;
use crate::object::{AudioSteganography, ResultStego, UniqueRandomIndices};
use derive_builder::Builder;
use std::path::{Path, PathBuf};

const LSB_DEEP_DEFAULT: u8 = 1;
const MAX_OCCUPANCY: usize = 70;
const HEADER: &str = "STEG";

const fn lsb_deep_default() -> u8 {
    LSB_DEEP_DEFAULT
}

/// A steganography encoder/decoder for 16-bit WAV audio files.
///
/// This struct provides methods for hiding and extracting messages in 16-bit WAV files.
///
/// # Examples
///
/// ```rust
/// # use stego_wave::{formats::wav::WAV16, AudioSteganography};
///  // Create a default WAV16 instance (lsb_deep = 1)
/// let wav16 = WAV16::default();
///  // Create WAV16 instance with custom lsb_dep
/// let wav16 =  WAV16::builder().lsb_deep(4).build().unwrap();
/// ```
#[derive(Builder, Debug, PartialEq)]
#[builder(build_fn(validate = "Self::validate"))]
#[builder(name = "WAV16Builder")]
pub struct WAV16 {
    #[builder(default = "lsb_deep_default()")]
    lsb_deep: u8,
}

impl Default for WAV16 {
    fn default() -> Self {
        WAV16::builder().build().unwrap()
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
    fn is_enough_samples(&self, msg_len: usize, samples_len: usize) -> ResultStego<()> {
        let msg_bits = msg_len * 8;
        let total_bits = samples_len * (self.lsb_deep as usize) * MAX_OCCUPANCY / 100;

        if msg_bits > total_bits {
            let required_bits = msg_bits * 100 / MAX_OCCUPANCY;
            let required_samples = required_bits / (self.lsb_deep as usize);
            return Err(StegoError::NotEnoughSamples(required_samples + 1));
        }

        Ok(())
    }

    #[inline(always)]
    fn get_full_message_bit(header: &[u8], message: &[u8], bit_index: usize) -> u8 {
        let header_len = header.len();
        let message_len = message.len();

        let byte_index = bit_index / 8;

        if byte_index < header_len {
            (header[byte_index] >> (7 - (bit_index % 8))) & 1
        } else if byte_index < header_len + message_len {
            (message[byte_index - header_len] >> (7 - (bit_index % 8))) & 1
        } else {
            0
        }
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

    fn validate_header(
        &self,
        samples: &[i16],
        indicates_iter: &mut UniqueRandomIndices,
    ) -> ResultStego<(u8, u8, Vec<u8>)> {
        let mask: i16 = self.get_mask();
        let mut header_bytes = Vec::with_capacity(HEADER.len());
        let mut after_header_buff = Vec::new();

        let (mut current_byte, mut bit_count) = (0_u8, 0_u8);
        let mut full_header = false;

        for sample_index in indicates_iter {
            let encoded = (samples[sample_index] & mask) as u16;

            for shift in (0..self.lsb_deep).rev() {
                let bit = ((encoded >> shift) & 1) as u8;
                current_byte = (current_byte << 1) | bit;
                bit_count += 1;

                if bit_count == 8 {
                    if full_header {
                        after_header_buff.push(current_byte);
                    } else {
                        header_bytes.push(current_byte);
                    }
                    current_byte = 0;
                    bit_count = 0;

                    if header_bytes.len() == HEADER.len() {
                        full_header = true;
                    }
                }
            }

            if full_header {
                break;
            }
        }
        println!("{:?}", header_bytes);

        if header_bytes != HEADER.as_bytes() {
            return Err(StegoError::IncorrectPassword);
        }
        Ok((current_byte, bit_count, after_header_buff))
    }
}

macro_rules! encode_bits_full {
    ($header:expr, $message:expr, $start_bit:expr, $bits_per_sample:expr) => {{
        let mut value: u16 = 0;
        let mut bit_pos = $start_bit;
        for _ in 0..$bits_per_sample {
            value <<= 1;
            let bit = WAV16::get_full_message_bit($header, $message, bit_pos);
            value |= bit as u16;
            bit_pos += 1;
        }
        (value as i16, bit_pos)
    }};
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
    /// # use stego_wave::{formats::wav::WAV16, AudioSteganography};
    /// let mut samples = vec![8; 1_000];
    /// let wav = WAV16::default();
    /// wav.hide_message_binary(&mut samples, "Test message", "_").unwrap();
    /// let res = wav.extract_message_binary(&samples, "_").unwrap();
    /// assert_eq!(res, "Test message");
    /// ```
    fn hide_message_binary(
        &self,
        samples: &mut [i16],
        message: &str,
        password: &str,
    ) -> ResultStego<()> {
        let header_bytes = HEADER.as_bytes();
        let message_bytes = message.as_bytes();

        let total_bytes = header_bytes.len() + message_bytes.len() + 1;
        let per_sample = self.lsb_deep as usize;

        self.is_enough_samples(total_bytes, samples.len())?;

        let mut bit_index = 0;
        let mask = !self.get_mask();
        let indices_iter = UniqueRandomIndices::new(samples.len(), password, MAX_OCCUPANCY);

        for sample_index in indices_iter {
            let (value, new_bit_index) =
                encode_bits_full!(header_bytes, message_bytes, bit_index, per_sample);
            bit_index = new_bit_index;
            samples[sample_index] = (samples[sample_index] & mask) | value;
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
    fn extract_message_binary(&self, samples: &[i16], password: &str) -> ResultStego<String> {
        let mut indices_iter = UniqueRandomIndices::new(samples.len(), password, MAX_OCCUPANCY);
        let mask: i16 = self.get_mask();

        let (mut current_byte, mut bit_count, buff) = self.validate_header(samples, &mut indices_iter)?;
        let mut result = String::from_utf8(buff).unwrap_or_default();

        for sample_index in indices_iter {
            let encoded = (samples[sample_index] & mask) as u16;

            for shift in (0..self.lsb_deep).rev() {
                let bit = ((encoded >> shift) & 1) as u8;
                current_byte = (current_byte << 1) | bit;
                bit_count += 1;

                if bit_count == 8 {
                    if current_byte == 0 {
                        return Ok(result);
                    }

                    result.push(current_byte as char);
                    current_byte = 0;
                    bit_count = 0;
                }
            }
        }
        Err(StegoError::FailedToReceiveMessage)
    }

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

    fn clear_secret_message_binary(&self, samples: &mut [i16], password: &str) -> ResultStego<()> {
        let indices_iter = UniqueRandomIndices::new(samples.len(), password, MAX_OCCUPANCY);
        let mask = self.get_mask();
        let (mut current_byte, mut bit_count, _) = self.validate_header(samples, &mut indices_iter.clone())?;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::error::Error;
    use std::fs;

    #[test]
    fn extract_message_binary_success() -> Result<(), Box<dyn Error>> {
        let sample: &mut [i16; 1_000] = &mut [8; 1_000];
        for i in 1..17 {
            println!("{i}");
            let wav16 = WAV16::builder().lsb_deep(i).build()?;

            wav16.hide_message_binary(sample, &format!("{i} test {i}"), "_")?;
            let res = wav16.extract_message_binary(sample, "_")?;
            assert_eq!(res, format!("{i} test {i}"));
        }

        Ok(())
    }

    #[test]
    fn extract_message_binary_failed() -> Result<(), Box<dyn Error>> {
        let sample: &mut [i16; 1_000] = &mut [8; 1_000];
        let wav16 = WAV16::default();
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

        let wav16 = WAV16::default();
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

        let wav16 = WAV16::default();
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

        let wav16 = WAV16::default();
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

        let res = WAV16::default().hide_message(&input_path, &output_path, "test", "rest");

        match res {
            Err(StegoError::InvalidFile(err)) => assert_eq!(err, "Only 16-bit WAV file supported"),
            _ => assert!(false),
        }

        let _ = fs::remove_file(input_path);
        let _ = fs::remove_file(output_path);

        Ok(())
    }
}