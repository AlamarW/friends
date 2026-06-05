use anyhow::Result;
use serde::Deserialize;

use crate::apps::AppletItem;

#[derive(Deserialize)]
struct WikiResponse {
    title: String,
    extract: String,
}

pub async fn fetch_summaries(interests: &[String]) -> Result<Vec<AppletItem>> {
    let client = reqwest::Client::new();
    let mut items = Vec::new();

    for interest in interests {
        let url = format!(
            "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
            urlencoding::encode(interest)
        );
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(data) = resp.json::<WikiResponse>().await {
                let subtitle = data.extract.chars().take(120).collect::<String>();
                let wiki_url = format!(
                    "https://en.wikipedia.org/wiki/{}",
                    urlencoding::encode(&data.title)
                );
                items.push(AppletItem {
                    title: data.title,
                    subtitle: Some(subtitle),
                    url: Some(wiki_url),
                });
            } else {
                items.push(AppletItem {
                    title: interest.clone(),
                    subtitle: Some("(No Wikipedia article found)".to_string()),
                    url: None,
                });
            }
        }
    }

    Ok(items)
}
