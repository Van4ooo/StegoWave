use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "sw",
    version,
    author,
    about = "StegoWave :: Audio file steganography🦀"
)]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) commands: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Hides a secret message in an audio file")]
    Hide(HideCommand),
    #[command(about = "Extracts a hidden secret message from an audio file")]
    Extract(ExtractCommand),
    #[command(about = "Clear the hidden secret message from an audio file")]
    Clear(ClearCommand),
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, ValueEnum)]
pub enum StegoWaveServer {
    GRPC,
    REST,
    Auto,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum StegoWaveFormat {
    WAV16,
}

impl From<StegoWaveFormat> for String {
    fn from(value: StegoWaveFormat) -> Self {
        match value {
            StegoWaveFormat::WAV16 => "wav16".to_string(),
        }
    }
}

#[derive(Debug, Parser)]
pub struct CommonFields {
    #[arg(
        long = "input_file",
        help = "Path to the input audio file from which bytes will be read"
    )]
    pub input_file: Option<PathBuf>,

    #[arg(
        value_enum,
        long = "format",
        short = 'f',
        help = "Audio file format (e.g., wav16, ...) used for processing the file"
    )]
    pub format: StegoWaveFormat,

    #[arg(
        value_enum,
        default_value = "auto",
        long = "server",
        short = 's',
        help = "The name of the server to be used"
    )]
    pub server: StegoWaveServer,

    #[arg(
        long = "lsb_deep",
        short = 'l',
        help = "Number of least significant bits to modify",
        default_value_t = 1
    )]
    pub lsb_deep: u8,
}

#[derive(Debug, Parser)]
pub struct HideCommand {
    #[arg(
        long = "message",
        short = 'm',
        help = "The secret message to hide inside the audio file"
    )]
    pub message: String,

    #[clap(flatten)]
    pub command: CommonFields,

    #[arg(
        long = "output_file",
        help = "Path where the audio file with the hidden text will be saved."
    )]
    pub output_file: Option<PathBuf>,
}

#[derive(Debug, Parser)]
pub struct ExtractCommand {
    #[clap(flatten)]
    pub command: CommonFields,
}

#[derive(Debug, Parser)]
pub struct ClearCommand {
    #[clap(flatten)]
    pub command: CommonFields,

    #[arg(
        long = "output_file",
        help = "Path where the audio file with the cleaned hidden text will be saved."
    )]
    pub output_file: Option<PathBuf>,
}
