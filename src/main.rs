mod film;

use clap::{error::ErrorKind, CommandFactory, Parser};

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

fn main() {
    let args = Args::parse();
    let file_path = args.file_path;

    if args.elo {
        unimplemented!("elo based ranking of films is not yet supported");
    }

    let films = match film::get_letterboxd_watched(&file_path) {
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

    for film in &films {
        println!("{:?}", film);
    }

    let f = &films[0];
    let f = f.extend();
    println!("{:?}", f);
}
