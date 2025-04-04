use crate::CONFIG_FILE;
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

impl Cli {
    pub fn get_command(&self) -> &Commands {
        &self.commands
    }
    pub fn get_server(&self) -> StegoWaveServer {
        match self.get_command() {
            Commands::Hide(hide) => hide.command.server.clone(),
            Commands::Extract(extract) => extract.command.server.clone(),
            Commands::Clear(clear) => clear.command.server.clone(),
        }
    }

    pub fn get_start_server(&self) -> bool {
        match self.get_command() {
            Commands::Hide(hide) => hide.command.start_server,
            Commands::Extract(extract) => extract.command.start_server,
            Commands::Clear(clear) => clear.command.start_server,
        }
    }
    pub fn get_file_config(&self) -> &str {
        match self.get_command() {
            Commands::Hide(hide) => &hide.command.config,
            Commands::Extract(extract) => &extract.command.config,
            Commands::Clear(clear) => &clear.command.config,
        }
    }
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
        long = "start-server",
        default_value_t = false,
        help = "Automatically start the server if it's not running"
    )]
    pub start_server: bool,

    #[arg(
        long = "lsb_deep",
        short = 'l',
        help = "Number of least significant bits to modify",
        default_value_t = 1
    )]
    pub lsb_deep: u8,

    #[arg(
        long = "config",
        help = "Specify the path to the configuration file",
        env = "SW_CONFIG",
        default_value = CONFIG_FILE,
    )]
    pub config: String,
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
