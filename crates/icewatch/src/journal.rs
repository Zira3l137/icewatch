use std::path::Path;
use std::path::PathBuf;

use chrono::DateTime;
use chrono::Local;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) enum ActionType {
    Automatic,
    Manual,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) enum Action {
    Moved { source: PathBuf, destination: PathBuf },
    Renamed { source: PathBuf, destination: PathBuf },
    Downloaded(PathBuf),
    Removed(PathBuf),
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct Entry {
    pub time: DateTime<Local>,
    pub action: Action,
    pub action_type: ActionType,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub(crate) struct Journal {
    pub(crate) entries: Vec<Entry>,
}

impl Journal {
    pub(crate) fn log(&mut self, action: Action, action_type: ActionType) {
        let entry = Entry { time: Local::now(), action, action_type };
        self.entries.push(entry);
    }

    pub(crate) fn entries_before(&self, time: DateTime<Local>) -> Vec<&Entry> {
        self.entries.iter().filter(|e| e.time <= time).collect()
    }

    #[expect(dead_code)]
    pub(crate) fn entries_after(&self, time: DateTime<Local>) -> Vec<&Entry> {
        self.entries.iter().filter(|e| e.time >= time).collect()
    }

    pub(crate) fn entries_between(
        &self,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Vec<&Entry> {
        self.entries.iter().filter(|e| e.time >= start && e.time <= end).collect()
    }

    pub(crate) fn filtered(&self, mut predicate: impl FnMut(&Path) -> bool) -> Self {
        self.entries
            .clone()
            .into_iter()
            .filter(|e| match &e.action {
                Action::Moved { source, destination } => {
                    predicate(&source) || predicate(&destination)
                }
                Action::Renamed { source, destination } => {
                    predicate(&source) || predicate(&destination)
                }
                Action::Removed(path) => predicate(&path),
                Action::Downloaded(path) => predicate(&path),
            })
            .collect()
    }
}

impl FromIterator<Entry> for Journal {
    fn from_iter<T: IntoIterator<Item = Entry>>(iter: T) -> Self {
        Self { entries: iter.into_iter().collect() }
    }
}
