use crate::error::StegoError;
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

    fn clear_secret_message_binary(&self, samples: &mut [i16], password: &str) -> ResultStego<()>;

    fn validate_file(&self, file: &Path) -> ResultStego<()>;
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

#[cfg(test)]
mod tests {
    use super::UniqueRandomIndices;

    fn inner_func(iter: &mut UniqueRandomIndices) {
        for x in iter {
            assert_eq!(x, 142);
            break;
        }
    }

    #[test]
    fn test_iterator() {
        let mut iter = UniqueRandomIndices::new(200, "_", 70);
        let ref_iter = &mut iter;

        for x in ref_iter {
            assert_eq!(x, 155);
            break;
        }

        inner_func(&mut iter);

        for i in iter {
            assert_eq!(i, 187);
            break;
        }
    }
}
