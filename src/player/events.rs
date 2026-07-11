use std::time::Duration;

#[derive(Debug)]
pub enum PlayerEvent {
    Started { duration: Duration },
    Progress { position: Duration, duration: Duration },
    Finished,
    Error(String),
}
