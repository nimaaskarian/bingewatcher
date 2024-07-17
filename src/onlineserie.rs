use error_chain::error_chain;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::{
    fmt::Display,
    io::{stdout, Write},
};

use crate::serie::{Season, Serie};

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
        Result(serde_json::Error);
    }
}

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

pub async fn online_tv_show(query: String) -> Result<()> {
    let main_response = request_pages(query.clone(), None).await?;
    let pages = main_response.pages;
    let mut handle = stdout().lock();
    write!(handle, "{main_response}")?;

    for i in 2..pages + 1 {
        if let Ok(response) = request_pages(query.clone(), Some(i)).await {
            write!(handle, "{response}")?;
        }
    }

    Ok(())
}

pub async fn request_pages(query: String, page: Option<usize>) -> Result<Response> {
    let page = page.unwrap_or(1);
    let target = format!("https://www.episodate.com/api/search?q={query}&page={page}");
    let response = reqwest::get(target).await?;
    let body = response.text().await?;
    let response: Response = serde_json::from_str(body.as_str())?;
    Ok(response)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TvShowDetails {
    episodes: Vec<EpisodeData>,
    name: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodeData {
    season: usize,
    episode: usize,
}

pub async fn request_detail(query: String) -> Result<Serie> {
    let target = format!("https://episodate.com/api/show-details?q={query}");
    let response = reqwest::get(target).await?;
    let body = response.text().await?;
    let details: Value = serde_json::from_str(body.as_str())?;
    let details: TvShowDetails = serde_json::from_value(details["tvShow"].clone())?;
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
    Ok(Serie::new(seasons, details.name))
}
