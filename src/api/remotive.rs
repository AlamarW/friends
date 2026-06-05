use anyhow::Result;
use serde::Deserialize;

use crate::apps::AppletItem;

#[derive(Deserialize)]
struct Job {
    title: String,
    company_name: String,
    #[serde(default)]
    candidate_required_location: String,
    url: String,
    #[serde(default)]
    salary: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Deserialize)]
struct RemotiveResponse {
    jobs: Vec<Job>,
}

pub async fn fetch_jobs(role: &str, skills: &[String]) -> Result<Vec<AppletItem>> {
    let url = format!(
        "https://remotive.com/api/remote-jobs?search={}&limit=30",
        urlencoding::encode(role)
    );

    let response: RemotiveResponse = reqwest::get(&url).await?.json().await?;

    let jobs = if !skills.is_empty() {
        let skills_lower: Vec<String> = skills.iter().map(|s| s.to_lowercase()).collect();
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

    Ok(jobs
        .into_iter()
        .map(|j| {
            let loc = if j.candidate_required_location.is_empty() {
                "Remote".to_string()
            } else {
                j.candidate_required_location
            };
            let subtitle = if j.salary.is_empty() {
                loc
            } else {
                format!("{loc} • {}", j.salary)
            };
            AppletItem {
                title: format!("{} — {}", j.title, j.company_name),
                subtitle: Some(subtitle),
                url: Some(j.url),
            }
        })
        .collect())
}
