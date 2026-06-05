use anyhow::Result;
use serde::Deserialize;

use crate::data::friend::JobHuntProfile;

#[derive(Debug, Clone, Deserialize)]
pub struct Job {
    pub title: String,
    pub company_name: String,
    pub candidate_required_location: String,
    pub url: String,
    #[serde(default)]
    pub salary: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
struct RemotiveResponse {
    jobs: Vec<Job>,
}

pub async fn fetch_jobs(profile: &JobHuntProfile) -> Result<Vec<Job>> {
    let url = format!(
        "https://remotive.com/api/remote-jobs?search={}&limit=30",
        urlencoding::encode(&profile.desired_role)
    );

    let response: RemotiveResponse = reqwest::get(&url).await?.json().await?;

    let jobs = if !profile.skills.is_empty() {
        let skills_lower: Vec<String> = profile.skills.iter().map(|s| s.to_lowercase()).collect();
        let mut scored: Vec<(usize, Job)> = response
            .jobs
            .into_iter()
            .map(|job| {
                let text = format!("{} {}", job.title, job.tags.join(" ")).to_lowercase();
                let score = skills_lower.iter().filter(|s| text.contains(s.as_str())).count();
                (score, job)
            })
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().map(|(_, j)| j).collect()
    } else {
        response.jobs
    };

    Ok(jobs)
}
