use anyhow::Result;
use serde::Deserialize;

use crate::apps::AppletItem;

#[derive(Deserialize)]
struct Doc {
    title: String,
    #[serde(default)]
    author_name: Vec<String>,
    first_publish_year: Option<i32>,
    #[serde(default)]
    key: String,
}

#[derive(Deserialize)]
struct SearchResponse {
    docs: Vec<Doc>,
}

pub async fn fetch_books(genres: &[String], authors: &[String]) -> Result<Vec<AppletItem>> {
    let client = reqwest::Client::new();
    let mut all_docs: Vec<Doc> = Vec::new();

    for genre in genres {
        let url = format!(
            "https://openlibrary.org/search.json?subject={}&limit=10&fields=title,author_name,first_publish_year,key",
            urlencoding::encode(genre)
        );
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(data) = resp.json::<SearchResponse>().await {
                all_docs.extend(data.docs);
            }
        }
    }

    for author in authors {
        let url = format!(
            "https://openlibrary.org/search.json?author={}&limit=10&fields=title,author_name,first_publish_year,key",
            urlencoding::encode(author)
        );
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(data) = resp.json::<SearchResponse>().await {
                all_docs.extend(data.docs);
            }
        }
    }

    all_docs.dedup_by(|a, b| a.title.to_lowercase() == b.title.to_lowercase());

    Ok(all_docs
        .into_iter()
        .map(|doc| {
            let authors_str = doc.author_name.join(", ");
            let year = doc.first_publish_year.map(|y| format!(" ({y})")).unwrap_or_default();
            let subtitle = if authors_str.is_empty() {
                None
            } else {
                Some(format!("{authors_str}{year}"))
            };
            AppletItem {
                title: doc.title,
                subtitle,
                url: Some(format!("https://openlibrary.org{}", doc.key)),
            }
        })
        .collect())
}
