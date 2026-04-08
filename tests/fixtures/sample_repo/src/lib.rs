mod auth;
mod db;

pub fn login(username: &str) -> bool {
    auth::login(username)
}
