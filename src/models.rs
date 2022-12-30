use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct UnsplashResponse {
    pub results: Vec<UnsplashResult>,
}

#[derive(Deserialize, Debug)]
pub struct UnsplashResult {
    pub width: u16,
    pub height: u16,
    pub urls: UnsplashUrls,
}

#[derive(Deserialize, Debug)]
pub struct UnsplashUrls {
    pub full: String,
    pub raw: String,
}
