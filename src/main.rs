use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use sentencex::segment;

use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};

use clap::{Parser, Subcommand};
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// sets file to parse
    #[arg(short, long, value_name = "FILE")]
    file_path: Option<String>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}
#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        #[arg(short, long)]
        list: bool,
    },
}
fn main() {
    let cli = Cli::parse();

    if let Some(config_path) = cli.file_path.as_deref() {
        println!("value for text transcript to parse: {}", config_path);
        match get_file_contents(config_path) {
            Ok(buf) => segment_sentences(buf.replace("\n", " ")),
            Err(err) => eprintln!("{err}"),
        }
    }

    if let Err(err) = test_rust_bert() {
        eprintln!("{err}");
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
fn test_rust_bert() -> anyhow::Result<()> {
    let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
        .create_model()?;

    let senteces = ["this is an exale sentence", "each sentence is converted"];

    let embeddings = model.encode(&senteces)?;

    println!("{embeddings:?}");
    Ok(())
}
