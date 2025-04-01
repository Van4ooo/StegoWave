use grpc_client::StegoWaveGrpcClient;
use grpc_server::startup::run_server;
use pretty_assertions::assert_eq;
use std::error::Error;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use stego_wave::error::StegoWaveClientError;
use stego_wave::object::StegoWaveClient;

fn create_wav_file(path: &PathBuf, bits: u16, samples: &[i16]) -> Result<(), Box<dyn Error>> {
    let spec_file = hound::WavSpec {
        channels: 1,
        sample_rate: 44_100,
        bits_per_sample: bits,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec_file)?;
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

async fn test_extract_message(
    client: &mut impl StegoWaveClient,
    result: Vec<u8>,
    valid_password: bool,
) {
    for (password, lsb_deep, success) in [
        ("qwerty1234", 1, valid_password),
        ("qwerty1234", 2, false),
        ("password", 1, false),
    ] {
        match client
            .extract_message(
                result.clone(),
                "input_file.wav".to_string(),
                password.to_string(),
                "wav16".to_string(),
                lsb_deep,
            )
            .await
        {
            Ok(message) => {
                assert!(success);
                assert_eq!("Secret Message", &message);
            }
            Err(StegoWaveClientError::Response(err)) => {
                assert!(!success);
                assert_eq!("Error password is incorrect", err.to_string());
            }
            _ => panic!(),
        }
    }
}

macro_rules! clear_message {
    ($client:expr, $result:expr, $password:expr) => {
        $client
            .clear_message(
                $result,
                "input_full.wav".to_string(),
                $password,
                "wav16".to_string(),
                1,
            )
            .await
    };
}

async fn full_test_client(
    mut client: impl StegoWaveClient,
    file_name: &str,
) -> Result<(), Box<dyn Error>> {
    let samples: Vec<i16> = vec![0; 10_000];
    let input_path = temp_path(file_name);
    create_wav_file(&input_path, 16, &samples)?;

    let file_byte = fs::read(&input_path)?;
    let result = client
        .hide_message(
            file_byte,
            "input_full.wav".to_string(),
            "Secret Message".to_string(),
            "qwerty1234".to_string(),
            "wav16".to_string(),
            1,
        )
        .await?;

    test_extract_message(&mut client, result.clone(), true).await;
    assert!(clear_message!(&mut client, result.clone(), "qwerty".to_string()).is_err());

    let result = clear_message!(&mut client, result.clone(), "qwerty1234".to_string())?;
    test_extract_message(&mut client, result, false).await;

    let _ = fs::remove_file(input_path);

    Ok(())
}

#[tokio::test]
async fn test_grpc_client() -> Result<(), Box<dyn Error>> {
    let settings = grpc_server::configuration::Settings::new("../../sw_config")?;
    let addr: SocketAddr = settings.address().parse()?;

    let _ = tokio::spawn(run_server(
        addr.clone(),
        settings,
    ));

    let addrs = format!("http://{}", addr);
    let client = StegoWaveGrpcClient::new(addrs.as_str()).await?;
    full_test_client(client, "grpc.wav").await?;
    Ok(())
}
