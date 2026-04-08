use crate::db::write_session;

pub fn login(username: &str) -> bool {
    write_session(username);
    true
}
