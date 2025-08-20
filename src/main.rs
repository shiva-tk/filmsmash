mod film;
mod ranker;
mod tui;

use clap::{error::ErrorKind, CommandFactory, Parser};
use ranker::Ranker;
use std::io;
use ratatui;

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

#[tokio::main]
async fn main() -> io::Result<()> {
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

    let mut terminal = ratatui::init();
    let ranker = Box::new(
        ranker::MergeSortRanker::new(films)
    );
    let mut tui = tui::Tui::new(ranker);
    let res = tui.run(&mut terminal).await;
    ratatui::restore();
    tui.print_top_10();
    tui.write_ranking();
    res
}
