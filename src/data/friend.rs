use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friend {
    #[serde(default = "new_id")]
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub interests: Vec<String>,
    pub job_hunt: Option<JobHuntProfile>,
    pub book_profile: Option<BookProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JobHuntProfile {
    pub desired_role: String,
    #[serde(default)]
    pub skills: Vec<String>,
    pub location: Option<String>,
    pub remote_preference: Option<String>,
    pub experience_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookProfile {
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub authors: Vec<String>,
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

impl Friend {
    pub fn new(name: String) -> Self {
        Self {
            id: new_id(),
            name,
            email: None,
            notes: None,
            interests: Vec::new(),
            job_hunt: None,
            book_profile: None,
        }
    }
}
