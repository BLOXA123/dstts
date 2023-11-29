use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::*;
use rodio::{source::Source, Decoder, OutputStream, OutputStreamHandle};
use serde_json::Value;
use serenity::{
    async_trait, framework::StandardFramework, model::channel::Message, prelude::*,
    Result as SerenityResult,
};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;
use tts_rust::{languages::Languages, tts::GTTSClient};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, msg: Message) {
        let name_string: String = get_from_config("nickname");
        let name: &str = &name_string;

        if msg.author.name == name || name.is_empty() {
            println!("{}: {}", msg.author.name, msg.content);
            play_audio(&msg.content);
        }
    }
}

#[tokio::main]
async fn main() -> SerenityResult<()> {
    if get_from_config("token").is_empty() {
        std::process::exit(0);
    }

    let _token_string: String = get_from_config("token");
    let _token: &str = &_token_string;

    let token = _token;

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(StandardFramework::new())
        .await
        .expect("Error creating client");

    client.start().await?;
    Ok(())
}

fn get_from_config(value: &str) -> String {
    let data = fs::read_to_string("Config.json").unwrap();
    let config: Value = serde_json::from_str(&data).unwrap();
    match config.get(value).and_then(Value::as_str) {
        Some(desired_value) => desired_value.to_string(),
        None => "".to_string(),
    }
}

fn play_audio(text: &str) {
    let language_string: String = get_from_config("language");
    let language: &str = &language_string;

    let narrator: GTTSClient = GTTSClient {
        volume: 1.0,
        language: Languages::from_str(language).unwrap(),
        tld: "com",
    };

    narrator.save_to_file(text, "Audio.mp3").unwrap();

    let (_stream, stream_handle) = get_output_stream("CABLE Input (VB-Audio Virtual Cable)");

    let file = BufReader::new(File::open("Audio.mp3").unwrap());
    let source = Decoder::new(file).unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    sink.append(source.convert_samples::<i16>());
    sink.sleep_until_end();
}

fn get_output_stream(device_name: &str) -> (OutputStream, OutputStreamHandle) {
    let host = cpal::default_host();
    let devices = host.output_devices().unwrap();
    let (mut _stream, mut stream_handle) = OutputStream::try_default().unwrap();
    for device in devices {
        let dev: rodio::Device = device.into();
        let dev_name: String = dev.name().unwrap();
        if dev_name == device_name {
            (_stream, stream_handle) = OutputStream::try_from_device(&dev).unwrap();
        }
    }
    (_stream, stream_handle)
}
