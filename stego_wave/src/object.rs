use crate::error::{StegoError, StegoWaveClientError};
use hound::WavSpec;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

pub type ResultStego<T> = Result<T, StegoError>;

pub trait AudioSteganography<S> {
    type Builder;

    fn builder() -> Self::Builder;

    fn hide_message(
        &self,
        file_input: impl Into<PathBuf>,
        file_output: impl Into<PathBuf>,
        message: impl Into<String>,
        password: impl Into<String>,
    ) -> ResultStego<()>;

    fn hide_message_binary(
        &self,
        samples: &mut [S],
        message: &str,
        password: &str,
    ) -> ResultStego<()>;

    fn extract_message(
        &self,
        file: impl Into<PathBuf>,
        password: impl Into<String>,
    ) -> ResultStego<String>;

    fn extract_message_binary(&self, samples: &[S], password: &str) -> ResultStego<String>;
    fn clear_secret_message(&self, file: impl Into<PathBuf>, password: &str) -> ResultStego<()>;
    fn clear_secret_message_binary(&self, samples: &mut [S], password: &str) -> ResultStego<()>;
    fn validate_file(&self, file: &Path) -> ResultStego<()>;
    fn read_samples_from_byte(&self, byte: Vec<u8>) -> ResultStego<(Vec<S>, AudioFileSpec)>;
    fn write_samples_to_byte(&self, spec: AudioFileSpec, samples: &[S]) -> ResultStego<Vec<u8>>;
    fn default_filename(&self) -> String;
}

pub enum AudioFileSpec {
    Wav(WavSpec),
}

#[derive(Clone)]
pub struct UniqueRandomIndices {
    rng: ChaCha8Rng,
    sample_len: usize,
    used: HashSet<usize>,
    max_count: usize,
    yielded: usize,
}

impl UniqueRandomIndices {
    pub fn new(sample_len: usize, password: &str, max_occupancy: usize) -> Self {
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);

        let seed = hasher.finish();
        let rng = ChaCha8Rng::seed_from_u64(seed);
        let max_count = (sample_len * max_occupancy) / 100;

        Self {
            rng,
            sample_len,
            used: HashSet::new(),
            max_count,
            yielded: 0,
        }
    }
}

impl Iterator for UniqueRandomIndices {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.yielded >= self.max_count {
            return None;
        }

        loop {
            let candidate = self.rng.gen_range(0..self.sample_len);
            if !self.used.contains(&candidate) {
                self.used.insert(candidate);
                self.yielded += 1;

                return Some(candidate);
            }
        }
    }
}

pub struct ByteIterator<'a, I, T> {
    samples: &'a [T],
    indices_iter: I,
    mask: i16,
    lsb_deep: u8,
    current_byte: u8,
    current_bit: u8,
    temp_buffer: Vec<u8>,
    temp_index: usize,
}

impl<'a, I, T> ByteIterator<'a, I, T>
where
    I: Iterator<Item = usize>,
{
    pub fn new(
        samples: &'a [T],
        indices_iter: I,
        mask: i16,
        lsb_deep: u8,
        current_byte: u8,
        current_bit: u8,
    ) -> Self {
        Self {
            samples,
            indices_iter,
            mask,
            lsb_deep,
            current_byte,
            current_bit,
            temp_buffer: Vec::new(),
            temp_index: 0,
        }
    }
}

impl<I> Iterator for ByteIterator<'_, I, i16>
where
    I: Iterator<Item = usize>,
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.temp_index < self.temp_buffer.len() {
            self.temp_index += 1;
            return Some(self.temp_buffer[self.temp_index - 1]);
        }

        self.temp_buffer.clear();
        self.temp_index = 0;

        let mut full_read = false;

        while let Some(sample_index) = self.indices_iter.next() {
            let encoded = (self.samples[sample_index] & self.mask) as u16;

            for shift in (0..self.lsb_deep).rev() {
                let bit = ((encoded >> shift) & 1) as u8;
                self.current_byte = (self.current_byte << 1) | bit;
                self.current_bit += 1;

                if self.current_bit == 8 {
                    self.temp_buffer.push(self.current_byte);
                    full_read = true;

                    self.current_byte = 0;
                    self.current_bit = 0;
                }
            }

            if full_read {
                return self.next();
            }
        }
        None
    }
}

#[async_trait::async_trait]
pub trait StegoWaveClient: Sync + Send {
    async fn hide_message(
        &mut self,
        file: Vec<u8>,
        message: String,
        password: String,
        format: String,
        lsb_deep: u8,
    ) -> Result<Vec<u8>, StegoWaveClientError>;

    async fn extract_message(
        &mut self,
        file: Vec<u8>,
        password: String,
        format: String,
        lsb_deep: u8,
    ) -> Result<String, StegoWaveClientError>;

    async fn clear_message(
        &mut self,
        file: Vec<u8>,
        password: String,
        format: String,
        lsb_deep: u8,
    ) -> Result<Vec<u8>, StegoWaveClientError>;
}

#[cfg(test)]
mod tests {
    use super::{ByteIterator, UniqueRandomIndices};

    fn inner_func(iter: &mut UniqueRandomIndices) {
        for x in iter {
            assert_eq!(x, 142);
            break;
        }
    }

    #[test]
    fn test_random_iterator() {
        let mut iter = UniqueRandomIndices::new(200, "_", 70);
        let ref_iter = &mut iter;

        for x in ref_iter {
            assert_eq!(x, 155);
            break;
        }

        inner_func(&mut iter);
    }

    #[test]
    fn test_single_byte() {
        let samples: Vec<i16> = vec![1, 0, 0, 1];
        let indices = (0..samples.len()).collect::<Vec<_>>().into_iter();

        let byte_iter = ByteIterator::new(&samples, indices, 3, 2, 0, 0);
        let result: Vec<u8> = byte_iter.collect();

        assert_eq!(result, vec![0x41]);
    }

    #[test]
    fn test_multiple_bytes() {
        let samples: Vec<i16> = vec![1, 0, 2, 0, 1, 2, 2, 1];
        let indices = (0..samples.len()).collect::<Vec<_>>().into_iter();

        let byte_iter = ByteIterator::new(&samples, indices, 3, 2, 0, 0);
        let result: Vec<u8> = byte_iter.collect();

        assert_eq!(result, vec![0x48, 0x69]);
    }
}
