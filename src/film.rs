use omdb::OmdbResponse;
use time::{Date, format_description::parse};
use std::{fmt::Display, io::{Cursor, Read}};
use zip::ZipArchive;
use csv::ReaderBuilder;
use std::fs;
use tokio::sync::OnceCell;

mod omdb {
    use once_cell::sync::Lazy;
    use reqwest::Client;

    static OMDB_CLIENT: Lazy<Client> = Lazy::new(|| {
        Client::new()
    });

    const OMDB_API_KEY: &str = "771c7c09";

    #[derive(Debug, serde::Deserialize)]
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

    pub async fn fetch_metadata(
        title: &str,
        year: u32,
    ) -> Result<OmdbResponse, Box<dyn std::error::Error>> {
        let url = format!(
            "http://www.omdbapi.com/?t={}&y={}&apikey={}",
            title, year, OMDB_API_KEY
        );

        let resp = OMDB_CLIENT.get(&url).send().await?;
        let json = resp.json::<OmdbResponse>().await?;
        Ok(json)
    }
}

#[derive(Debug)]
pub struct Film {
    name: String,
    year: u32,
    date_watched: Date,
    metadata: OnceCell<OmdbResponse>
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
            date_watched,
            metadata: OnceCell::new()
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

    pub async fn genre(&self) -> Option<&String> {
        self.metadata().await.Genre.as_ref()
    }

    pub async fn director(&self) -> Option<&String> {
        self.metadata().await.Director.as_ref()
    }

    pub async fn plot(&self) -> Option<&String> {
        self.metadata().await.Plot.as_ref()
    }

    async fn metadata(&self) -> &OmdbResponse {
        self.metadata
            .get_or_init(|| async {
                // only one API call ever made per Film
                omdb::fetch_metadata(&self.name, self.year)
                    .await
                    .unwrap() // handle errors properly in real code
            })
            .await
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
