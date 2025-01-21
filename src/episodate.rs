use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::{
    fmt::Display,
    io::{self, Write},
    result,
};

use crate::serie::{Season, Serie};

struct PageError;
type PageResult<T> = result::Result<T, PageError>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    pages: usize,
    tv_shows: Vec<TvShow>,
    is_selected: Option<bool>,
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for tv_show in &self.tv_shows {
            writeln!(f, "{tv_show}")?
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TvShow {
    name: String,
    start_date: Option<String>,
    permalink: String,
}

impl Display for TvShow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.permalink)
    }
}

pub async fn search(query: String) {
    if let Ok(main_response) = request_pages(query.clone(), None).await {
        let pages = main_response.pages;
        let mut handle = io::stdout().lock();
        let _ = write!(handle, "{main_response}");

        for i in 2..pages + 1 {
            if let Ok(response) = request_pages(query.clone(), Some(i)).await {
                let _ = write!(handle, "{response}");
            }
        }
    }
}

pub async fn request_detail(permalink: &str) -> Serie {
    let target = format!("https://episodate.com/api/show-details?q={permalink}");
    let response = reqwest::get(target).await.expect("Error sending get request");
    let body = response.text().await.expect("Error reading the resonse text");
    let mut details: Value = serde_json::from_str(body.as_str()).expect("Error converting json");
    let details: TvShowDetails = serde_json::from_value(std::mem::take(&mut details["tvShow"])).expect("Error converting to show details");
    let mut last_season = 0;
    let mut seasons: Vec<Season> = vec![];
    for episode in details.episodes {
        if last_season != episode.season {
            last_season = episode.season;
            while seasons.len() < last_season {
                seasons.push(Season::new(0));
            }
        }
        seasons[last_season - 1].episodes += 1;
    }
    Serie::new(seasons, details.name)
}

async fn request_pages(query: String, page: Option<usize>) -> PageResult<Response> {
    let page = page.unwrap_or(1);
    let target = format!("https://www.episodate.com/api/search?q={query}&page={page}");
    if let Ok(response) = reqwest::get(target).await {
        if let Some(response) = response.text().await.ok().and_then(|body| serde_json::from_str(body.as_str()).ok()).flatten() {
            return Ok(response)
        }
    }
    Err(PageError)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TvShowDetails {
    episodes: Vec<EpisodeData>,
    name: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct EpisodeData {
    season: usize,
    episode: usize,
}
