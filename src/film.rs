use time::{Date, format_description::parse};
use std::{fmt::Display, io::{Cursor, Read}};
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

    pub fn fetch_metadata(title: &str, year: u32) -> Result<OmdbResponse, Box<dyn std::error::Error>> {
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
    name: String,
    year: u32,
    genre: Option<String>,
    director: Option<String>,
    plot: Option<String>,
    date_watched: Date,
    metadata_fetched: bool
}

impl Display for Film {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Film {
    pub fn new(name: &str, year: u32, date_watched: Date) -> Film {
        Film {
            name: name.to_string(),
            year,
            genre: None,
            director: None,
            plot: None,
            date_watched,
            metadata_fetched: false
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn year(&self) -> u32 {
        self.year
    }

    pub fn date_watched(&self) -> Date {
        self.date_watched
    }

    pub fn genre(&mut self) -> Option<&str> {
        self.ensure_metadata();
        self.genre.as_deref()
    }

    pub fn director(&mut self) -> Option<&str> {
        self.ensure_metadata();
        self.director.as_deref()
    }

    pub fn plot(&mut self) -> Option<&str> {
        self.ensure_metadata();
        self.plot.as_deref()
    }

    fn ensure_metadata(&mut self) {
        if !self.metadata_fetched {
            self.fetch_metadata();
        }
    }

    fn fetch_metadata(&mut self) {
        let resp = omdb::fetch_metadata(&self.name, self.year);
        if let Ok(r) = resp {
            if r.Response == "True" {
                self.genre = r.Genre;
                self.director = r.Director;
                self.plot = r.Plot;
            }
        }
        self.metadata_fetched = true;
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
