use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use sentencex::segment;

use ytt::YouTubeTranscript;

use clap::{Parser, Subcommand, ValueEnum};
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// sets file to parse
    #[arg(short, long, value_name = "FILE")]
    file_path: Option<String>,
    /// sets youtube_link to fetch
    #[arg(short, long, value_name = "YOUTUBELINK")]
    link: Option<String>,

    #[arg(short, long, value_enum, value_name = "MODEL")]
    model: Option<Model>,
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Model {
    /// Model type Kimi-k2.5:cloud
    KimiK2,
    /// Model type Qwen3.5:cloud
    Qwen3,
    /// Model type glm-5:cloud
    Glm5,
}
#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        #[arg(short, long)]
        list: bool,
    },
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(config_path) = cli.file_path.as_deref() {
        match get_file_contents(config_path) {
            Ok(buf) => {
                segment_sentences(buf.replace("\n", " "));

                let ollama = Ollama::default();

                let model = "kimi-k2.5:cloud".to_string();
                let prompt = buf;

                let res = ollama.generate(GenerationRequest::new(model, prompt)).await;

                match res {
                    Ok(res) => println!("{}", res.response),
                    Err(err) => eprintln!("{err}"),
                }
            }
            Err(err) => eprintln!("{err}"),
        }
    }
    if let Some(youtube_link) = cli.link.as_deref() {
        let api = YouTubeTranscript::new();
        let video_id = YouTubeTranscript::extract_video_id(youtube_link)?;
        let transcript = api.fetch_transcript(&video_id, Some(vec!["en"])).await?;
        let mut buf = String::new();
        for item in transcript.transcript {
            buf += &(item.text + " ");
        }

        let ollama = Ollama::default();

        let mut chosen_model = String::new();
        if let Some(model) = cli.model.as_ref() {
            match model {
                Model::KimiK2 => chosen_model = "kimi-k2.5:cloud".to_string(),
                Model::Qwen3 => chosen_model = "qwen3.5:cloud".to_string(),
                Model::Glm5 => chosen_model = "glm-5:cloud".to_string(),
            }
        } else {
            chosen_model = "kimi-k2.5:cloud".to_string();
        }
        let prompt = buf
            + "This is a youtube transcript, turn it into a readable article. Maintain the authors style.";

        let res = ollama
            .generate(GenerationRequest::new(chosen_model, prompt))
            .await;

        match res {
            Ok(res) => println!("{}", res.response),
            Err(err) => eprintln!("{err}"),
        }
    }

    match &cli.command {
        Some(Commands::Test { list }) => {
            if *list {
                println!("Printing testing lists...");
            } else {
                print!("not printing testing lists...");
            }
        }
        None => {}
    }

    Ok(())
}
fn get_file_contents(file_path: &str) -> io::Result<String> {
    let path = Path::new(file_path);
    let mut f = File::open(path)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    Ok(buf)
}
// pass in a file/transcript to read
fn segment_sentences(text: String) {
    let senteces = segment("en", &text);
    for (i, sentence) in senteces.iter().enumerate() {
        println!("{}. {}", i + 1, sentence);
    }
}
