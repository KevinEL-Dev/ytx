use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;
use std::{fs, process::exit};
use std::{io, process};
use std::collections::HashMap;

use directories::ProjectDirs;

use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;

use rusqlite::{Connection, Result, named_params};

use regex::Regex;

use sentencex::segment;

use ytt::{TranscriptResponse, YouTubeTranscript};

use clap::{Parser, Subcommand, ValueEnum};
#[derive(Parser)]
#[command(version, about, long_about = None)]
/// A command line utility that generates articles from youtube videos.
struct Cli {
    // /// sets file to parse
    // #[arg(short, long, value_name = "FILE")]
    // file_path: Option<String>,
    /// sets url to fetch
    #[arg(value_name = "url")]
    link: Option<String>,
    /// sets ollama model to generate readable transcript
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
    /// Local Model glm-4.7-flash
    Glm4flash,
}
#[derive(Subcommand)]
enum Commands {
    /// List out saved transcripts
    List,
    /// Open a transcript where identifier is either an Id or Article title
    Open {
        #[arg(value_parser = get_identifier)]
        identifier: Identifier,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Identifier {
    Id(i32),
    Title(String),
}
#[derive(Debug, Clone)]
struct Transcript {
    video_id: i32,
    title: String,
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
        if !res
            && let Some(data_path) = return_data_dir("ytx".to_string())
            && let Err(err) = create_dir_for_cli(data_path)
        {
            eprint!(
                "something went wrong in creating the dir for our favorite cli tool. err: {err}"
            );
            process::exit(1);
        }
    } else {
        println!("something went wrong in getting xdg directories");
        process::exit(1);
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
            panic!("we failed to create the video table");
        }
    }
    // then create transcript tables
    let res_for_transcript_table =
        check_if_tables_exist(&con, "transcript").expect("failed to check table");
    if !res_for_transcript_table {
        create_table_transcript(&con, "transcript").expect("failet to add table transcript");
    }
    // create ai transcript tables
    let res_for_ai_transcript_table = check_if_tables_exist(&con, "ai_transcript")
        .expect("failed to check table for ai_transcript");
    if !res_for_ai_transcript_table {
        create_table_ai_transcript(&con, "ai_transcript")
            .expect("failet to add table ai_transcript");
    }
    // some sort of checking to see if lama installed
    if !check_if_ollama_installed() {
        eprint!("ollama not installed, install ollama please");
        panic!();
    }

    // if let Some(config_path) = cli.file_path.as_deref() {
    //     match get_file_contents(config_path) {
    //         Ok(buf) => {
    //             segment_sentences(buf.replace("\n", " "));
    //
    //             let ollama = Ollama::default();
    //
    //             let model = "kimi-k2.5:cloud".to_string();
    //             let prompt = buf;
    //
    //             let res = ollama.generate(GenerationRequest::new(model, prompt)).await;
    //
    //             match res {
    //                 Ok(res) => println!("{}", res.response),
    //                 Err(err) => eprintln!("{err}"),
    //             }
    //         }
    //         Err(err) => eprintln!("{err}"),
    //     }
    // }
    if let Some(youtube_link) = cli.link.as_deref() {
        let vid_id = parse_vid_id_from_youtube_link(youtube_link.to_string().clone());
        // before we fetch using the api lets check our database for the video
        //
        // if the video exist within our database, lets check if we have the transcript for it
        match check_if_video_exist_in_video_table(&con, vid_id.clone()) {
            Ok(row) => {
                // if the video exist within our database, lets check if we have the transcript for it
                // use the video_id to search the transcript database
                match fetch_ai_transcript_body_using_video_id(&con, row) {
                    Ok(body) => {
                        // im thinking here maybe we can prompt the user
                        // if they would like regenerate another propmt instead
                        println!("{body}");
                        process::exit(0);
                    }
                    Err(_err) => {
                        println!(
                            "this should not happen because where there exist a video, a ai transcript exist"
                        );
                    }
                }
            }
            // we havent seen this youtube video before
            Err(_err) => {
                let api = YouTubeTranscript::new();
                let video_id = YouTubeTranscript::extract_video_id(youtube_link)?;
                let transcript = api.fetch_transcript(&video_id, Some(vec!["en"])).await?;
                let mut buf = String::new();
                for item in transcript.transcript.clone() {
                    buf += &(item.text + " ");
                }
                // check before inserting new video
                match check_if_video_exist_in_video_table(&con, vid_id.clone()) {
                    Ok(row) => println!("{row}, not a new video, so we wont insert a new video"),
                    Err(_err) => {
                        insert_new_video_via_link(&con, youtube_link.to_string().clone())
                            .expect("failed to insert a value");
                    }
                }
                // create the transcript text for video if it dont not exist
                if let Err(_err) = check_if_transcript_exists_in_transcript_table(&con, &transcript)
                {
                    insert_new_transcript_for_vid_id(
                        &con,
                        buf.clone(),
                        vid_id.clone(),
                        &transcript,
                    )
                    .expect("failed to insert transcript");
                } else {
                    println!("transcript exist so lets not add");
                }

                let ollama = Ollama::default();

                let mut chosen_model = String::new();
                if let Some(model) = cli.model.as_ref() {
                    match model {
                        Model::KimiK2 => chosen_model = "kimi-k2.5:cloud".to_string(),
                        Model::Qwen3 => chosen_model = "qwen3.5:cloud".to_string(),
                        Model::Glm5 => chosen_model = "glm-5:cloud".to_string(),
                        Model::Glm4flash => chosen_model = "glm-4.7-flash".to_string(),
                    }
                } else {
                    chosen_model = "kimi-k2.5:cloud".to_string();
                }
                let prompt = buf
                    + "This is a youtube transcript, turn it into a readable article. Maintain the authors style.";

                // how can i do one of those loading bars when complete?
                println!("Generating article...");
                let res = ollama
                    .generate(GenerationRequest::new(chosen_model, prompt))
                    .await;

                let transcript_id =
                    check_if_transcript_exists_in_transcript_table(&con, &transcript)
                        .expect("we failed to get transcript id in main before ai check");
                // here lets check if we have generated an ai_transcript already
                match res {
                    Ok(res) => {
                        if let Err(_err) = check_if_ai_transcript_exists_in_ai_transcript_table(
                            &con,
                            transcript_id,
                        ) {
                            insert_new_ai_generated_transcript_for_vid_id(
                                &con,
                                res.response.to_string().clone(),
                                vid_id,
                                &transcript,
                            )
                            .expect("failed to insert new ai_transcript");
                            println!("{}", res.response);
                        } else {
                            // probably wont reach here
                            println!("ai_transcript exist so lets not add");
                        }
                    }
                    // most likely an error with either auth for cloud models or local models not
                    // installed
                    Err(err) => eprintln!("{err}"),
                }
            }
        }
    }

    match &cli.command {
        Some(Commands::List) => {
            if let Err(err) = get_all_videos(&con) {
                println!("something went wrong fetching your transcripts");
                eprintln!("{err}");
            }
        }
        Some(Commands::Open { identifier }) => {
            match identifier {
                Identifier::Id(i) => match get_transcript_body_from_video_id(&con, *i) {
                    Ok(body) => println!("{}", body),
                    Err(err) => println!("did not find a transcript with the id. {err}"),
                },
                // little tricky, not exactly sure how searching via title will go, will return the
                // most similar title to what the user entered?
                Identifier::Title(title) => {
                    if let Err(err) = get_transcript_body_from_title(&con, title.to_string()) {
                        eprintln!("{err}");
                    };
                }
            }
        }
        None => {}
    }
    Ok(())
}
fn _get_file_contents(file_path: &str) -> io::Result<String> {
    let path = Path::new(file_path);
    let mut f = File::open(path)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    Ok(buf)
}
// pass in a file/transcript to read
fn _segment_sentences(text: String) {
    let senteces = segment("en", &text);
    for (i, sentence) in senteces.iter().enumerate() {
        println!("{}. {}", i + 1, sentence);
    }
}
// this needs fixing as which can be different across platforms
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
    //println!("{}", db.is_autocommit());
    Ok(db)
}
fn check_if_tables_exist(con: &Connection, table_name: &str) -> Result<bool> {
    // two tables should exist
    // youtube video id table
    let res_video_table = con
        .table_exists(Some("main"), table_name)
        .expect("something went wrong searching tables");
    Ok(res_video_table)
}
fn create_table_video(con: &Connection, table_name: &str) -> Result<()> {
    let sql = "Create TABLE ".to_owned()
        + table_name
        + "(id INTEGER PRIMARY KEY,
        video_id TEXT NOT NULL UNIQUE,
        video_link TEXT NOT NULL UNIQUE);";
    match con.execute(&sql, ()) {
        Ok(_updated) => Ok(()),
        Err(err) => {
            println!("update failed: {}", err);
            process::exit(1);
        }
    }
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
        Ok(_updated) => Ok(()),
        Err(err) => {
            println!("update failed: {}", err);
            process::exit(1);
        }
    }
}
fn create_table_ai_transcript(con: &Connection, table_name: &str) -> Result<()> {
    let sql = "Create TABLE ".to_owned()
        + table_name
        + "(id INTEGER PRIMARY KEY,
        video_id INTEGER REFERENCES video(id),
        transcript_id INTEGER REFERENCES transrcipt(id),
        body TEXT NOT NULL,
        language NEXT NOT NULL);";
    match con.execute(&sql, ()) {
        Ok(_updated) => Ok(()),
        Err(err) => {
            println!("update failed: {}", err);
            process::exit(1);
        }
    }
}
// insert via link
fn insert_new_video_via_link(con: &Connection, video_link: String) -> Result<()> {
    // parse video link for vid id
    let vid_id = parse_vid_id_from_youtube_link(video_link.clone());
    let sql = "INSERT INTO video (video_id, video_link)
        VALUES(:video_id,:video_link);";
    match con.execute(sql, &[(":video_id", &vid_id), (":video_link", &video_link)]) {
        Ok(_updated) => Ok(()),
        Err(err) => {
            println!("insert failed in for new video: {}", err);
            process::exit(1);
        }
    }
}
fn insert_new_transcript_for_vid_id(
    con: &Connection,
    body: String,
    vid_id: String,
    transcript_response: &TranscriptResponse,
) -> Result<()> {
    let vid_id_int = check_if_video_exist_in_video_table(con, vid_id)
        .expect("failed to get vid id in transcript function");
    let title = transcript_response
        .title
        .clone()
        .expect("failed to get title");
    let language = transcript_response.language.clone();
    let sql = "INSERT INTO transcript (video_id, title, body,language)
        VALUES(:video_id,:title,:body,:language);";
    match con.execute(
        sql,
        named_params! {
            ":video_id": vid_id_int,
            ":title": title,
            ":body": body,
            ":language": language,
        },
    ) {
        Ok(_updated) => Ok(()),
        Err(err) => {
            println!("update failed in insert transcript: {}", err);
            process::exit(1);
        }
    }
}
fn insert_new_ai_generated_transcript_for_vid_id(
    con: &Connection,
    body: String,
    vid_id: String,
    transcript_response: &TranscriptResponse,
) -> Result<()> {
    let vid_id_int = check_if_video_exist_in_video_table(con, vid_id)
        .expect("failed to get vid id in transcript function");
    let transcript_id_int =
        check_if_transcript_exists_in_transcript_table(con, transcript_response)
            .expect("failed to get trascript id in ai_transcript function");
    let language = transcript_response.language.clone();
    let sql = "INSERT INTO ai_transcript (video_id,transcript_id, body,language)
        VALUES(:video_id,:transcript_id,:body,:language);";
    match con.execute(
        sql,
        named_params! {
            ":video_id": vid_id_int,
            ":transcript_id": transcript_id_int,
            ":body": body,
            ":language": language,
        },
    ) {
        Ok(_updated) => Ok(()),
        Err(err) => {
            println!("update failed in insert ai_transcript: {}", err);
            process::exit(1);
        }
    }
}
fn parse_vid_id_from_youtube_link(video_link: String) -> String {
    // video link will be https://www.youtube.com/watch?v=<vid_id>
    let re = Regex::new(r"https://www.youtube.com/watch\?v=(.{11})").unwrap();
    let hay = &video_link;
    let caps = re.captures(hay).unwrap();
    caps[1].to_string()
}
fn check_if_video_exist_in_video_table(con: &Connection, vid_id: String) -> Result<i32> {
    con.query_row(
        "SELECT id FROM video WHERE video_id = :video_id",
        named_params! {":video_id":&vid_id},
        |row| row.get::<_, i32>(0),
    )
}
fn check_if_transcript_exists_in_transcript_table(
    con: &Connection,
    transcript_response: &TranscriptResponse,
) -> Result<i32> {
    let title = transcript_response
        .title
        .clone()
        .expect("failed to get title");
    let language = transcript_response.language.clone();
    con.query_row(
        "SELECT id FROM transcript WHERE title = :title AND language = :language",
        named_params! {":title":title,":language":language},
        |row| row.get::<_, i32>(0),
    )
}
fn check_if_ai_transcript_exists_in_ai_transcript_table(
    con: &Connection,
    transcript_id: i32,
) -> Result<i32> {
    con.query_row(
        "SELECT id FROM ai_transcript WHERE transcript_id = :transcript_id",
        named_params! {":transcript_id":transcript_id},
        |row| row.get::<_, i32>(0),
    )
}
fn _check_if_ai_transcript_exists_in_ai_transcript_table_via_vid_id(
    con: &Connection,
    video_id: i32,
) -> Result<i32> {
    con.query_row(
        "SELECT id FROM ai_transcript WHERE video_id = :video_id",
        named_params! {":video_id":video_id},
        |row| row.get::<_, i32>(0),
    )
}
fn fetch_ai_transcript_body_using_video_id(con: &Connection, video_id: i32) -> Result<String> {
    con.query_row(
        "SELECT body FROM ai_transcript WHERE video_id = :video_id",
        named_params! {":video_id":video_id},
        |row| row.get::<_, String>(0),
    )
}
fn get_all_videos(con: &Connection) -> Result<()> {
    let mut stmt = con.prepare("SELECT title, video_id FROM transcript")?;
    let transcript_iter = stmt.query_map([], |row| {
        Ok(Transcript {
            title: row.get(0)?,
            video_id: row.get(1)?,
        })
    })?;
    let mut in_order_video_id_mappings: HashMap<i32,i32> = HashMap::new();
    let mut collect: Vec<Transcript> = Vec::new();
    let mut counter = 1;
    for transcript in transcript_iter {
        let handled_transcript = transcript.unwrap();
        collect.push(handled_transcript.clone());
        in_order_video_id_mappings.insert(counter,handled_transcript.video_id);
        println!(
            "{}  {}",
            counter, handled_transcript.title
        );
        counter += 1;
    }
    Ok(())
}
fn get_mappings_for_videos(con: &Connection) -> Result<HashMap<i32,i32>> {
    let mut stmt = con.prepare("SELECT title, video_id FROM transcript")?;
    let transcript_iter = stmt.query_map([], |row| {
        Ok(Transcript {
            title: row.get(0)?,
            video_id: row.get(1)?,
        })
    })?;
    let mut in_order_video_id_mappings: HashMap<i32,i32> = HashMap::new();
    let mut collect: Vec<Transcript> = Vec::new();
    let mut counter = 1;
    for transcript in transcript_iter {
        let handled_transcript = transcript.unwrap();
        collect.push(handled_transcript.clone());
        in_order_video_id_mappings.insert(counter,handled_transcript.video_id);
        counter += 1;
    }
    Ok(in_order_video_id_mappings)
}
fn get_identifier(s: &str) -> Result<Identifier, String> {
    let identifier_int = s.trim().parse::<i32>();
    match identifier_int {
        Ok(int) => Ok(Identifier::Id(int)),
        Err(_err) => Ok(Identifier::Title(s.to_string())),
    }
}
fn get_transcript_body_from_video_id(con: &Connection, video_id: i32) -> Result<String> {
    let mappings = get_mappings_for_videos(con).expect("failed to get mappings for transcripts");
    match mappings.get(&video_id) {
        Some(mapped_video_id) => {
            con.query_row(
                "SELECT body FROM ai_transcript WHERE video_id = :video_id",
                named_params! {":video_id":mapped_video_id},
                |row| row.get::<_, String>(0),
            )
        }
        None => Ok("Invalid video id passed".to_string())
    }
}
fn get_transcript_body_from_title(con: &Connection, title: String) -> Result<()> {
    let mut stmt = con.prepare("SELECT title, video_id FROM transcript WHERE title LIKE :title")?;
    let title_param = format!("%{}%", title);
    let title_iter = stmt.query_map(named_params! {":title":title_param}, |row| {
        Ok(Transcript {
            title: row.get(0)?,
            video_id: row.get(1)?,
        })
    })?;
    let mut collect: Vec<Transcript> = Vec::new();
    for transcript in title_iter {
        let handled_transcript = transcript.unwrap();
        collect.push(handled_transcript.clone());
    }
    if collect.len() == 1 {
        match get_transcript_body_from_video_id(con, collect[0].video_id) {
            Ok(body) => println!("{}", body),
            Err(err) => println!("did not find a transcript with the id. {err}"),
        }
    } else {
        println!("Multiple videos found please select one");
        let mut counter = 1;
        for transcript in &collect {
            println!("{}  {}", counter, transcript.title);
            counter += 1;
        }
        println!("Choose a video id: ");
        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("failed to read your input");
        let parsed_user_choice = choice.trim().parse::<i32>().unwrap();
        match get_transcript_body_from_video_id(con, parsed_user_choice) {
            Ok(body) => println!("{}", body),
            Err(err) => println!("did not find a transcript with the id. {err}"),
        }
    }
    Ok(())
}
