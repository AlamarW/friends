use crate::data::friend::Friend;

#[derive(Debug, Clone, PartialEq)]
pub enum AppletKind {
    JobFeed,
    BookRecs,
    WikiPrep,
}

impl AppletKind {
    pub fn label(&self) -> &str {
        match self {
            AppletKind::JobFeed => "Job Feed",
            AppletKind::BookRecs => "Book Recs",
            AppletKind::WikiPrep => "Conversation Prep",
        }
    }
}

pub fn available_applets(friend: &Friend) -> Vec<AppletKind> {
    let mut applets = Vec::new();

    if let Some(jh) = &friend.job_hunt {
        if !jh.desired_role.is_empty() {
            applets.push(AppletKind::JobFeed);
        }
    }

    if let Some(bp) = &friend.book_profile {
        if !bp.genres.is_empty() || !bp.authors.is_empty() {
            applets.push(AppletKind::BookRecs);
        }
    }

    if !friend.interests.is_empty() {
        applets.push(AppletKind::WikiPrep);
    }

    applets
}
