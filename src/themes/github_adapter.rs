use std::f32::consts::E;
use serde::Deserialize;



const GITHUB_BASE_URL: &str = "https://api.github.com/repos";
const GITHUB_RATE_LIMIT: &str = "https://api.github.com/rate_limit";

pub struct AdapterOptions {
    repository: String,
    directory: String
}

impl AdapterOptions {
    
    pub fn new(repository: String, directory: String) -> Self {
        Self { repository, directory }
    }

    pub fn get_base_url(&self) -> String {
        format!("{GITHUB_BASE_URL}/{}/contents/{}", self.repository, self.directory)
    }

}


pub struct GithubAdapter {
    options: AdapterOptions,
    client: reqwest::blocking::Client
}

impl GithubAdapter {
    
    pub fn new(options: AdapterOptions) -> Self {
        Self { options, client: reqwest::blocking::Client::new() }
    }

    pub fn get_rate_limit(&self) -> Result<RateLimitResponse, reqwest::Error> {

        let response = self.client
            .get(GITHUB_RATE_LIMIT)
            .header("User-Agent", "GithubAdapter-Rate-Limit-checker")
            .send()?;

        if !response.status().is_success() {
            return Err(reqwest::Error::from(response.error_for_status().err().unwrap()));
        }

        let rate_limit: RateLimitResponse = response.json()?;

        Ok(rate_limit) 

    }

    pub fn list_dir(&self) -> Result<Vec<GitHubContent>, reqwest::Error> {

        let response = self.client
            .get(&self.options.get_base_url())
            .header("User-Agent", "GithubAdapter")
            .send()?;

        if !response.status().is_success() {
            return Err(reqwest::Error::from(response.error_for_status().err().unwrap()));
        }

        let contents: Vec<GitHubContent> = response.json()?;

        Ok(contents)

    }

}





#[derive(Deserialize)]
pub struct GitHubContent {
    pub name: String,
    pub r#type: String, // "file" or "dir"
}

#[derive(Deserialize)]
pub struct RateLimitResponse {
    pub rate: Rate,
}

#[derive(Deserialize)]
pub struct Rate {
    pub limit: u32,       // Total limit for the current window
    pub remaining: u32,   // Requests left
    pub reset: u64,       // Time when the rate limit resets (UNIX timestamp)
}