use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

use directories::ProjectDirs;

use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use rusqlite::{Connection, Result, params};
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_data_dir() {
        let app_name = "test".to_string();
        match return_data_dir(app_name.clone()) {
            Some(test_path) => {
                // create the test directory in xdg path data directory
                create_dir_for_cli(test_path.clone()).expect("failed to create test directory");
                // if test directory is created now check if it exist
                let exists = check_if_data_dir_exist(app_name.clone())
                    .expect("failed to check if data dir exists");

                // if exists is false, it will print the message
                assert!(exists, "data diretory should exist");
                // if we made it here than yay, lets remove the test dir that we created
                remove_dir(test_path).expect("failed to remove our test directory");
            }
            None => panic!("something went wrong getting test path"),
        }
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    if let Some(res) = check_if_data_dir_exist("ytx".to_string()) {
        match res {
            true => println!("the path exists for our data lets go"),
            // we should create the directory  for our user
            false => {
                println!("the path does not exist, welcome new user");
                if let Some(data_path) = return_data_dir("ytx".to_string()) {
                    if let Err(err) = create_dir_for_cli(data_path) {
                        eprint!(
                            "something went wrong in creating the dir for our favorite cli tool. err: {err}"
                        );
                    } else {
                        println!("yay we made the directory maybe")
                    }
                }
            }
        }
    } else {
        println!("something went wrong in getting xdg directories")
    }
    // dir for cli should be created so lets create a connection to our db
    let app_name_path =
        return_data_dir("ytx".to_string()).expect("failed to retrieve app data dir");

    let con = open_ytx_db(app_name_path).expect("failed to open connection to db");

    // once we get the connection lets check if our tables are created
    let res_for_video_table = check_if_tables_exist(&con, "video").expect("failed to check table");
    if !res_for_video_table {
        create_table_video(&con, "video").expect("failed to add table video");
        let res_for_video_table =
            check_if_tables_exist(&con, "video").expect("failed to check table");
        if !res_for_video_table {
            panic!("we failed to create the table name");
        }
    }
    // then create transcript tables
    let res_for_transcript_table =
        check_if_tables_exist(&con, "transcript").expect("failed to check table");
    if !res_for_transcript_table {
        create_table_transcript(&con, "transcript").expect("failet to add table transcript");
        let res_for_video_table =
            check_if_tables_exist(&con, "transcript").expect("failed to check table");
        if !res_for_video_table {
            panic!("we failed to create the table transcript");
        }
    }
    // some sort of checking to see if lama installed
    if !check_if_ollama_installed() {
        eprint!("ollama not installed, install ollama please");
        panic!();
    }

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
fn check_if_ollama_installed() -> bool {
    let output = Command::new("which")
        .arg("ollama")
        .output()
        .expect("failed to execute which command");

    let mut string_output = String::new();
    if let Err(err) = output.stdout.as_slice().read_to_string(&mut string_output) {
        eprint!("error with reading output from which {err}");
    }
    if string_output.is_empty() || string_output == "ollama not found" {
        return false;
    }
    true
}
fn check_if_data_dir_exist(app_name: String) -> Option<bool> {
    if let Some(proj_dir) = ProjectDirs::from("", "", &app_name) {
        println!("{:?}", proj_dir.config_dir());
        return Some(fs::metadata(proj_dir.config_dir()).is_ok());
    }
    None
}
fn return_data_dir(app_name: String) -> Option<String> {
    if let Some(proj_dir) = ProjectDirs::from("", "", &app_name) {
        return Some(proj_dir.config_dir().to_str()?.to_string());
    }
    None
}
fn create_dir_for_cli(dir_path: String) -> std::io::Result<()> {
    fs::create_dir(dir_path)?;
    Ok(())
}
fn remove_dir(dir_path: String) -> std::io::Result<()> {
    fs::remove_dir(dir_path)?;
    Ok(())
}
fn open_ytx_db(path: String) -> Result<Connection> {
    let new_path = path + "/ytx.db";
    let db = Connection::open(new_path)?;
    println!("{}", db.is_autocommit());
    Ok(db)
}
fn check_if_tables_exist(con: &Connection, table_name: &str) -> Result<bool> {
    // two tables should exist
    // youtube video id table
    let res_video_table = con
        .table_exists(Some("main"), table_name)
        .expect("something went wrong searching tables");
    Ok(res_video_table)
    // if dne then return false
    // if youtubevideo_table dne, then create it
    // youtube metadata table that stores, trascript, language,
    // if youtube metadata dne, create it
}
fn create_table_video(con: &Connection, table_name: &str) -> Result<()> {
    let sql = "Create TABLE ".to_owned()
        + table_name
        + "(id INTEGER PRIMARY KEY,
        video_id TEXT NOT NULL UNIQUE,
        video_link TEXT NOT NULL UNIQUE);";
    match con.execute(&sql, ()) {
        Ok(updated) => println!("{} rows were updated", updated),
        Err(err) => println!("update failed: {}", err),
    }
    Ok(())
}
fn create_table_transcript(con: &Connection, table_name: &str) -> Result<()> {
    let sql = "Create TABLE ".to_owned()
        + table_name
        + "(id INTEGER PRIMARY KEY,
        video_id INTEGER REFERENCES video(id),
        title TEXT NOT NULL,
        body TEXT NOT NULL,
        language NEXT NOT NULL);";
    match con.execute(&sql, ()) {
        Ok(updated) => println!("{} rows were updated", updated),
        Err(err) => println!("update failed: {}", err),
    }
    Ok(())
}
