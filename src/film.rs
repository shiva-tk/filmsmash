use clap::Error;
use time::{Date, format_description::parse};
use std::io::{Cursor, Read};
use zip::ZipArchive;
use csv::ReaderBuilder;
use std::fs;

mod omdb {
    use reqwest::blocking::get;
    use serde::Deserialize;

    const OMDB_API_KEY: &str = "771c7c09";

    #[derive(Debug, Deserialize)]
    pub struct OmdbResponse {
        pub Title: Option<String>,
        pub Year: Option<String>,
        pub Genre: Option<String>,
        pub Director: Option<String>,
        pub Actors: Option<String>,
        pub Plot: Option<String>,
        pub imdbRating: Option<String>,
        pub Response: String,
        pub Error: Option<String>,
    }

    pub fn fetch_movie(title: &str, year: u32) -> Result<OmdbResponse, Box<dyn std::error::Error>> {
        let url = format!(
            "http://www.omdbapi.com/?t={}&y={}&apikey={}",
            title, year, OMDB_API_KEY
        );

        let resp: OmdbResponse = get(&url)?.json()?;
        Ok(resp)
    }
}

#[derive(Debug)]
pub struct Film {
    pub name: String,
    pub year: u32,
    pub date_watched: Date,
}

impl Film {
    pub fn new(name: &str, year: u32, date_watched: Date) -> Film {
        Film {
            name: name.to_string(),
            year,
            date_watched
        }
    }

    pub fn extend(&self) -> Result<ExtendedFilm, Box<dyn std::error::Error>> {
        ExtendedFilm::new(&self.name, self.year, self.date_watched)
    }
}

#[derive(Debug)]
pub struct ExtendedFilm {
    pub name: String,
    pub year: u32,
    pub genre: String,
    pub director: String,
    pub plot: String,
    pub date_watched: Date,
}

impl ExtendedFilm {
    pub fn new(name: &str, year: u32, date_watched: Date) -> Result<ExtendedFilm, Box<dyn std::error::Error>> {
        let resp = omdb::fetch_movie(&name, year)?;

        if resp.Response == "True" {
            Ok(ExtendedFilm {
                name: resp.Title.unwrap_or(name.to_string()),
                year,
                genre: resp.Genre.unwrap_or_else(|| "Unknown".to_string()),
                director: resp.Director.unwrap_or_else(|| "Unknown".to_string()),
                plot: resp.Plot.unwrap_or_else(|| "".to_string()),
                date_watched,
            })
        } else {
            Err(format!("OMDb error: {:?}", resp.Error).into())
        }
    }
}

pub fn get_letterboxd_watched(data_file_path: &str) -> Result<Vec<Film>, Box<dyn std::error::Error>> {
    let bytes = fs::read(data_file_path)?;
    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader)?;

    let mut file = archive.by_name("watched.csv")?;
    let mut csv_bytes = Vec::new();
    file.read_to_end(&mut csv_bytes)?;

    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_bytes.as_slice());

    let mut films = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let date_watched_format = parse("[year]-[month]-[day]")?;
        let date_watched = Date::parse(&record[0], &date_watched_format)?;
        let name = &record[1];
        let year : u32 = record[2].parse()?;
        let film = Film::new(name, year, date_watched);
        films.push(film);
    }

    Ok(films)
}
