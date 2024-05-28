use reqwest;
use error_chain::error_chain;
use json::{self, JsonValue};

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
        JsonNull(json::Error);
    }
}

#[tokio::main]
pub async fn request(query: String) -> Result<JsonValue> {
    let target = format!("https://www.episodate.com/api/search?q={query}&page=1");
    let response = reqwest::get(target).await?;
    let body = response.text().await?;
    if let Ok(json_body) = json::parse(body.as_str()) {
        let pages = json_body["pages"].to_string().parse::<u32>().unwrap();
        // sscanf("{}", json_body["pages"]);
        // println!("{}", json_body["pages"]);
        // return Ok(json_body);
        // json_body["pages"]
    }
    Ok(json::Null)
}
