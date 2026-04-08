use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiphyImage {
    pub url: String,
    pub width: String,
    pub height: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiphyImages {
    pub fixed_height: GiphyImage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiphyData {
    pub id: String,
    pub title: String,
    pub images: GiphyImages,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiphyResponse {
    pub data: Vec<GiphyData>,
}

pub struct GiphyService {
    api_key: String,
}

impl GiphyService {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<GiphyData>, String> {
        if query.is_empty() {
            return self.trending().await;
        }

        let url = format!(
            "https://api.giphy.com/v1/gifs/search?api_key={}&q={}&limit=20&rating=g",
            self.api_key,
            urlencoding::encode(query)
        );

        self.fetch_from_url(&url).await
    }

    pub async fn trending(&self) -> Result<Vec<GiphyData>, String> {
        let url = format!(
            "https://api.giphy.com/v1/gifs/trending?api_key={}&limit=20&rating=g",
            self.api_key
        );
        web_sys::console::log_1(&format!("Fetching Trending Giphy: {}", url).into());

        self.fetch_from_url(&url).await
    }

    async fn fetch_from_url(&self, url: &str) -> Result<Vec<GiphyData>, String> {
        let response = reqwest::get(url)
            .await
            .map_err(|e| format!("Failed to fetch Giphy: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Giphy API returned error: {}", response.status()));
        }

        let giphy_res: GiphyResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Giphy response: {}", e))?;

        Ok(giphy_res.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_giphy_response_deserialization() {
        let json = r#"{
            "data": [
                {
                    "id": "1",
                    "title": "Test GIF",
                    "images": {
                        "fixed_height": {
                            "url": "https://test.com/gif1",
                            "width": "200",
                            "height": "200"
                        }
                    }
                }
            ]
        }"#;
        let res: GiphyResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.data.len(), 1);
        assert_eq!(res.data[0].id, "1");
        assert_eq!(res.data[0].images.fixed_height.url, "https://test.com/gif1");
    }
}
