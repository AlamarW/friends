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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::friend::{BookProfile, Friend, JobHuntProfile};

    fn bare_friend() -> Friend {
        Friend::new("Alice".to_string())
    }

    fn friend_with_job(role: &str) -> Friend {
        let mut f = bare_friend();
        f.job_hunt = Some(JobHuntProfile {
            desired_role: role.to_string(),
            ..Default::default()
        });
        f
    }

    fn friend_with_books(genres: &[&str], authors: &[&str]) -> Friend {
        let mut f = bare_friend();
        f.book_profile = Some(BookProfile {
            genres: genres.iter().map(|s| s.to_string()).collect(),
            authors: authors.iter().map(|s| s.to_string()).collect(),
        });
        f
    }

    fn friend_with_interests(interests: &[&str]) -> Friend {
        let mut f = bare_friend();
        f.interests = interests.iter().map(|s| s.to_string()).collect();
        f
    }

    #[test]
    fn test_empty_friend_has_no_applets() {
        assert!(available_applets(&bare_friend()).is_empty());
    }

    #[test]
    fn test_job_feed_unlocks_with_nonempty_role() {
        let applets = available_applets(&friend_with_job("Engineer"));
        assert!(applets.contains(&AppletKind::JobFeed));
    }

    #[test]
    fn test_job_feed_does_not_unlock_with_empty_role() {
        let applets = available_applets(&friend_with_job(""));
        assert!(!applets.contains(&AppletKind::JobFeed));
    }

    #[test]
    fn test_book_recs_unlocks_with_genres() {
        let applets = available_applets(&friend_with_books(&["sci-fi"], &[]));
        assert!(applets.contains(&AppletKind::BookRecs));
    }

    #[test]
    fn test_book_recs_unlocks_with_authors_only() {
        let applets = available_applets(&friend_with_books(&[], &["Le Guin"]));
        assert!(applets.contains(&AppletKind::BookRecs));
    }

    #[test]
    fn test_book_recs_does_not_unlock_when_both_empty() {
        let applets = available_applets(&friend_with_books(&[], &[]));
        assert!(!applets.contains(&AppletKind::BookRecs));
    }

    #[test]
    fn test_wiki_prep_unlocks_with_interests() {
        let applets = available_applets(&friend_with_interests(&["climbing"]));
        assert!(applets.contains(&AppletKind::WikiPrep));
    }

    #[test]
    fn test_wiki_prep_does_not_unlock_with_empty_interests() {
        let applets = available_applets(&bare_friend());
        assert!(!applets.contains(&AppletKind::WikiPrep));
    }

    #[test]
    fn test_all_three_applets_unlock_together() {
        let mut f = friend_with_job("Engineer");
        f.book_profile = Some(BookProfile {
            genres: vec!["sci-fi".to_string()],
            authors: vec![],
        });
        f.interests = vec!["climbing".to_string()];

        let applets = available_applets(&f);
        assert_eq!(applets.len(), 3);
        assert!(applets.contains(&AppletKind::JobFeed));
        assert!(applets.contains(&AppletKind::BookRecs));
        assert!(applets.contains(&AppletKind::WikiPrep));
    }

    #[test]
    fn test_applet_labels_are_correct() {
        assert_eq!(AppletKind::JobFeed.label(), "Job Feed");
        assert_eq!(AppletKind::BookRecs.label(), "Book Recs");
        assert_eq!(AppletKind::WikiPrep.label(), "Conversation Prep");
    }

    #[test]
    fn test_job_feed_no_book_profile_present() {
        let applets = available_applets(&friend_with_job("Engineer"));
        assert!(!applets.contains(&AppletKind::BookRecs));
        assert!(!applets.contains(&AppletKind::WikiPrep));
    }
}
