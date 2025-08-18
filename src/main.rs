mod film;
mod ranker;

use clap::{error::ErrorKind, CommandFactory, Parser};
use ranker::Ranker;
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(name = "filmsmash")]
#[command(about = "A tool to rank your favourite films on Letterboxd.")]
#[command(long_about = Some("A tool to rank your favourite films on Letterboxd.\n\nTo use this tool, download your Letterboxd data from https://letterboxd.com/settings/data/"))]
struct Args {
    /// Path to the Letterboxd data export zip file
    file_path: String,

    /// Use ELO ranking instead of performing a merge sort
    #[arg(short, long)]
    elo: bool,
}

fn prompt(message: &str) -> String {
    let mut input = String::new();

    print!("{}", message);
    io::stdout().flush().unwrap(); // flush to ensure prompt shows

    io::stdin()
        .read_line(&mut input)
        .expect("failed to read line");

    input.trim().to_string() // strip trailing newline
}

fn main() {
    let args = Args::parse();
    let file_path = args.file_path;

    if args.elo {
        unimplemented!("elo based ranking of films is not yet supported");
    }

    let mut films = match film::get_letterboxd_watched(&file_path) {
        Ok(fs) => fs,
        Err(_) => {
            let mut cmd = Args::command();
            cmd.error(
                ErrorKind::Io,
                format!(
                    "failed to open and read letterboxd data at `{file_path}`\n\t"
                )
            ).exit()
        }
    };

    let mut film_ranker = ranker::MergeSortRanker::new(films.into_iter().take(10).collect());
    while !film_ranker.is_ranked() {
        let left = film_ranker.left().unwrap();
        let right = film_ranker.right().unwrap();

        println!("Left: {}", left.name());
        println!("Right: {}", right.name());

        let mut choice = prompt(">>> ");
        while (choice.trim() != "l" && choice.trim() != "r") {
            println!("{}", choice);
            println!("Pick your favourite film! Please enter `l` to choose the left film, and `r` to choose the right one.");
            choice = prompt(">>> ");
        }

        if choice.trim() == "l" {
            film_ranker.gt()
        } else {
            film_ranker.lt()
        }
    }

    let ranking = film_ranker.into_ranking().unwrap();
    for (i, f) in ranking.iter().enumerate() {
        println!("{}. {}", i, f.name())
    }
}
