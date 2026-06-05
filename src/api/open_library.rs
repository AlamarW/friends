use anyhow::Result;
use serde::Deserialize;

use crate::data::friend::BookProfile;

#[derive(Debug, Clone)]
pub struct Book {
    pub title: String,
    pub authors: Vec<String>,
    pub first_publish_year: Option<i32>,
    pub key: String,
}

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

pub async fn fetch_books(profile: &BookProfile) -> Result<Vec<Book>> {
    let mut all_books: Vec<Book> = Vec::new();
    let client = reqwest::Client::new();

    for genre in &profile.genres {
        let url = format!(
            "https://openlibrary.org/search.json?subject={}&limit=10&fields=title,author_name,first_publish_year,key",
            urlencoding::encode(genre)
        );
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(data) = resp.json::<SearchResponse>().await {
                for doc in data.docs {
                    all_books.push(Book {
                        title: doc.title,
                        authors: doc.author_name,
                        first_publish_year: doc.first_publish_year,
                        key: doc.key,
                    });
                }
            }
        }
    }

    for author in &profile.authors {
        let url = format!(
            "https://openlibrary.org/search.json?author={}&limit=10&fields=title,author_name,first_publish_year,key",
            urlencoding::encode(author)
        );
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(data) = resp.json::<SearchResponse>().await {
                for doc in data.docs {
                    all_books.push(Book {
                        title: doc.title,
                        authors: doc.author_name,
                        first_publish_year: doc.first_publish_year,
                        key: doc.key,
                    });
                }
            }
        }
    }

    // Deduplicate by title
    all_books.dedup_by(|a, b| a.title.to_lowercase() == b.title.to_lowercase());

    Ok(all_books)
}
