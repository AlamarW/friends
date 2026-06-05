use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct WikiSummary {
    pub title: String,
    pub extract: String,
}

#[derive(Deserialize)]
struct WikiResponse {
    title: String,
    extract: String,
}

pub async fn fetch_summaries(interests: &[String]) -> Result<Vec<WikiSummary>> {
    let client = reqwest::Client::new();
    let mut summaries = Vec::new();

    for interest in interests {
        let url = format!(
            "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
            urlencoding::encode(interest)
        );
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(data) = resp.json::<WikiResponse>().await {
                summaries.push(WikiSummary {
                    title: data.title,
                    extract: data.extract,
                });
            } else {
                summaries.push(WikiSummary {
                    title: interest.clone(),
                    extract: "(No Wikipedia article found)".to_string(),
                });
            }
        }
    }

    Ok(summaries)
}
