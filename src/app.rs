use crate::api::open_library::Book;
use crate::api::remotive::Job;
use crate::api::wikipedia::WikiSummary;
use crate::apps::AppletKind;
use crate::data::friend::Friend;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    FriendsList,
    FriendDetail,
    EditFriend,
    AddFriend,
    AppletView(AppletKind),
    ConfirmDelete,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadState<T> {
    Idle,
    Loading,
    Loaded(T),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditField {
    Name,
    Email,
    Notes,
    Interests,
    JobRole,
    JobSkills,
    JobLocation,
    JobRemote,
    JobLevel,
    BookGenres,
    BookAuthors,
}

impl EditField {
    pub fn all() -> Vec<EditField> {
        vec![
            EditField::Name,
            EditField::Email,
            EditField::Notes,
            EditField::Interests,
            EditField::JobRole,
            EditField::JobSkills,
            EditField::JobLocation,
            EditField::JobRemote,
            EditField::JobLevel,
            EditField::BookGenres,
            EditField::BookAuthors,
        ]
    }

    pub fn label(&self) -> &str {
        match self {
            EditField::Name => "Name",
            EditField::Email => "Email",
            EditField::Notes => "Notes",
            EditField::Interests => "Interests (comma-sep)",
            EditField::JobRole => "Job: Desired Role",
            EditField::JobSkills => "Job: Skills (comma-sep)",
            EditField::JobLocation => "Job: Location",
            EditField::JobRemote => "Job: Remote Pref (remote/hybrid/onsite)",
            EditField::JobLevel => "Job: Level (junior/mid/senior)",
            EditField::BookGenres => "Books: Genres (comma-sep)",
            EditField::BookAuthors => "Books: Authors (comma-sep)",
        }
    }
}

pub struct EditState {
    pub fields: Vec<(EditField, String)>,
    pub cursor: usize,
}

impl EditState {
    pub fn from_friend(friend: &Friend) -> Self {
        let jh = friend.job_hunt.as_ref();
        let bp = friend.book_profile.as_ref();
        let fields = vec![
            (EditField::Name, friend.name.clone()),
            (EditField::Email, friend.email.clone().unwrap_or_default()),
            (EditField::Notes, friend.notes.clone().unwrap_or_default()),
            (EditField::Interests, friend.interests.join(", ")),
            (EditField::JobRole, jh.map(|j| j.desired_role.clone()).unwrap_or_default()),
            (EditField::JobSkills, jh.map(|j| j.skills.join(", ")).unwrap_or_default()),
            (EditField::JobLocation, jh.and_then(|j| j.location.clone()).unwrap_or_default()),
            (EditField::JobRemote, jh.and_then(|j| j.remote_preference.clone()).unwrap_or_default()),
            (EditField::JobLevel, jh.and_then(|j| j.experience_level.clone()).unwrap_or_default()),
            (EditField::BookGenres, bp.map(|b| b.genres.join(", ")).unwrap_or_default()),
            (EditField::BookAuthors, bp.map(|b| b.authors.join(", ")).unwrap_or_default()),
        ];
        Self { fields, cursor: 0 }
    }

    pub fn blank() -> Self {
        let fields = EditField::all()
            .into_iter()
            .map(|f| (f, String::new()))
            .collect();
        Self { fields, cursor: 0 }
    }

    pub fn get(&self, field: &EditField) -> &str {
        self.fields
            .iter()
            .find(|(f, _)| f == field)
            .map(|(_, v)| v.as_str())
            .unwrap_or("")
    }

    pub fn get_mut(&mut self, field: &EditField) -> &mut String {
        self.fields
            .iter_mut()
            .find(|(f, _)| f == field)
            .map(|(_, v)| v)
            .unwrap()
    }

    pub fn apply_to_friend(&self, friend: &mut Friend) {
        friend.name = self.get(&EditField::Name).to_string();
        let email = self.get(&EditField::Email).trim().to_string();
        friend.email = if email.is_empty() { None } else { Some(email) };
        let notes = self.get(&EditField::Notes).trim().to_string();
        friend.notes = if notes.is_empty() { None } else { Some(notes) };
        friend.interests = parse_csv(self.get(&EditField::Interests));

        let job_role = self.get(&EditField::JobRole).trim().to_string();
        if !job_role.is_empty() {
            let jh = friend.job_hunt.get_or_insert_with(Default::default);
            jh.desired_role = job_role;
            jh.skills = parse_csv(self.get(&EditField::JobSkills));
            let loc = self.get(&EditField::JobLocation).trim().to_string();
            jh.location = if loc.is_empty() { None } else { Some(loc) };
            let remote = self.get(&EditField::JobRemote).trim().to_string();
            jh.remote_preference = if remote.is_empty() { None } else { Some(remote) };
            let level = self.get(&EditField::JobLevel).trim().to_string();
            jh.experience_level = if level.is_empty() { None } else { Some(level) };
        } else {
            friend.job_hunt = None;
        }

        let genres = parse_csv(self.get(&EditField::BookGenres));
        let authors = parse_csv(self.get(&EditField::BookAuthors));
        if !genres.is_empty() || !authors.is_empty() {
            let bp = friend.book_profile.get_or_insert_with(Default::default);
            bp.genres = genres;
            bp.authors = authors;
        } else {
            friend.book_profile = None;
        }
    }
}

fn parse_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

pub struct AppState {
    pub screen: Screen,
    pub friends: Vec<Friend>,
    pub selected_friend: usize,
    pub selected_applet: usize,
    pub edit: Option<EditState>,
    pub jobs: LoadState<Vec<Job>>,
    pub books: LoadState<Vec<Book>>,
    pub wiki: LoadState<Vec<WikiSummary>>,
    pub applet_scroll: usize,
    pub status_msg: Option<String>,
}

impl AppState {
    pub fn new(friends: Vec<Friend>) -> Self {
        Self {
            screen: Screen::FriendsList,
            friends,
            selected_friend: 0,
            selected_applet: 0,
            edit: None,
            jobs: LoadState::Idle,
            books: LoadState::Idle,
            wiki: LoadState::Idle,
            applet_scroll: 0,
            status_msg: None,
        }
    }

    pub fn current_friend(&self) -> Option<&Friend> {
        self.friends.get(self.selected_friend)
    }

    pub fn current_friend_mut(&mut self) -> Option<&mut Friend> {
        self.friends.get_mut(self.selected_friend)
    }
}
