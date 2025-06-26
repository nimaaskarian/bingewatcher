use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::{
    fmt::Display,
    io::{self, Write},
    result,
};
use reqwest::blocking::Client;

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

pub fn search_write_to_stdout(query: &str) {
    if let Ok(main_response) = request_pages(query, None) {
        let pages = main_response.pages;
        let mut handle = io::stdout().lock();
        let _ = write!(handle, "{main_response}");

        for i in 2..pages + 1 {
            if let Ok(response) = request_pages(query, Some(i)) {
                let _ = write!(handle, "{response}");
            }
        }
    }
}

pub fn request_detail(permalink: &str) -> Serie {
    let target = format!("https://episodate.com/api/show-details?q={permalink}");
    let response = Client::new().get(target).send().expect("Error sending get request");
    let body = response.text().expect("Error reading the resonse text");
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

fn request_pages(query: &str, page: Option<usize>) -> PageResult<Response> {
    let page = page.unwrap_or(1);
    let target = format!("https://www.episodate.com/api/search?q={query}&page={page}");
    if let Ok(response) = Client::new().get(target).send() {
        if let Some(response) = response.text().ok().and_then(|body| serde_json::from_str(body.as_str()).ok()).flatten() {
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
    name: String,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_page_detail_breaking_bad() {
        let serie = request_detail("breaking-bad");
        let expected = Serie::new(vec![
            Season::new(7),
            Season::new(13),
            Season::new(13),
            Season::new(13),
            Season::new(16),
            Season::new(8),
        ], "Breaking Bad");
        assert_eq!(serie, expected)
    }

    #[test]
    fn test_page_detail_peaky_blinders() {
        let serie = request_detail("peaky-blinders");
        let expected = Serie::new(vec![
            Season::new(6),
            Season::new(6),
            Season::new(6),
            Season::new(6),
            Season::new(6),
            Season::new(6),
        ], "Peaky Blinders");
        assert_eq!(serie, expected)
    }

    #[test]
    fn test_page_detail_person_of_interest() {
        let serie = request_detail("person-of-interest");
        let expected = Serie::new(vec![
            Season::new(23),
            Season::new(22),
            Season::new(23),
            Season::new(22),
            Season::new(13),
        ], "Person of Interest");
        assert_eq!(serie, expected)
    }
}
