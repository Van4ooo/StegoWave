use pretty_assertions::assert_eq;
use rest_client::StegoWaveRestClient;
use rest_server::configuration::Settings;
use rest_server::startup::run_server;
use std::error::Error;
use std::fs;
use std::net::TcpListener;
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
            .clear_message($result, $password, "wav16".to_string(), 1)
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
async fn test_rest_client() -> Result<(), Box<dyn Error>> {
    let mut settings = Settings::new("../../sw_config.toml").expect("Failed to read configuration");
    let stego_wave_setting = settings.get_stego_wave_lib_settings();

    let listener = TcpListener::bind(format!("{}:0", &settings.rest.host))
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port().clone();
    let addrs = format!("http://{}:{}", settings.rest.host, port);

    let server = run_server(listener, stego_wave_setting).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    let client = StegoWaveRestClient::new(addrs.as_str()).await?;
    full_test_client(client, "rest.wav").await?;
    Ok(())
}
