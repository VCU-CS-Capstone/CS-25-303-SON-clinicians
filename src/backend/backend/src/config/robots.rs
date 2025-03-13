use std::fmt::Display;

use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RobotsConfig {
    pub robots: Vec<RobotItem>,
}
impl RobotsConfig {
    pub fn response(&self) -> axum::response::Response {
        let body = self.to_string();
        http::Response::builder()
            .header(http::header::CONTENT_TYPE, "text/plain")
            .body(body.into())
            .unwrap()
    }
}
impl IntoResponse for RobotsConfig {
    fn into_response(self) -> axum::response::Response {
        let body = format!("{}", self);
        http::Response::builder()
            .header(http::header::CONTENT_TYPE, "text/plain")
            .body(body.into())
            .unwrap()
    }
}
impl Display for RobotsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for robot in &self.robots {
            write!(f, "{}\n", robot)?;
        }
        Ok(())
    }
}
impl Default for RobotsConfig {
    fn default() -> Self {
        Self {
            robots: vec![RobotItem {
                user_agent: "*".to_string(),
                disallow: vec!["/".to_string()],
                allow: vec![],
            }],
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RobotItem {
    pub user_agent: String,
    pub disallow: Vec<String>,
    pub allow: Vec<String>,
}
impl Display for RobotItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User-Agent: {}\n", self.user_agent)?;
        for disallow in &self.disallow {
            write!(f, "Disallow: {}\n", disallow)?;
        }
        for allow in &self.allow {
            write!(f, "Allow: {}\n", allow)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::robots::RobotsConfig;

    #[test]
    fn test_robots_config_default() {
        let config = RobotsConfig::default();
        let expected = "User-Agent: *\nDisallow: /\n\n";
        assert_eq!(expected, format!("{}", config));
    }
}
