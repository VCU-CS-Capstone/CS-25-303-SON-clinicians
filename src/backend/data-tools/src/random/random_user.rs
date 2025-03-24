#![allow(dead_code)]
use cs25_303_core::red_cap::Gender;
use serde::Deserialize;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum RandomUserError {
    #[error(transparent)]
    Rewqest(#[from] reqwest::Error),
}
#[derive(Debug, Deserialize)]
pub enum RandomUserGender {
    #[serde(rename = "female")]
    Female,
    #[serde(rename = "male")]
    Male,
}
impl From<RandomUserGender> for Gender {
    fn from(value: RandomUserGender) -> Self {
        match value {
            RandomUserGender::Female => Gender::Female,
            RandomUserGender::Male => Gender::Male,
        }
    }
}
#[derive(Debug, serde::Deserialize)]
pub struct RandomUserAPIResponse {
    pub results: Vec<RandomUser>,
    pub info: RandomUserAPIInfo,
}
#[derive(Debug, serde::Deserialize)]
pub struct RandomUser {
    pub name: RandomUserName,
    pub phone: String,
    pub gender: RandomUserGender,
}
#[derive(Debug, serde::Deserialize)]
pub struct RandomUserName {
    pub first: String,
    pub last: String,
    pub title: String,
}
impl RandomUserName {
    pub fn is_name_english_alphabet(&self) -> bool {
        self.first
            .chars()
            .all(|x| x.is_ascii_alphabetic() || x == '-')
            && self
                .last
                .chars()
                .all(|x| x.is_ascii_alphabetic() || x == '-')
    }
}
#[derive(Debug, serde::Deserialize)]
pub struct RandomUserAPIInfo {
    pub seed: String,
    pub results: usize,
    pub page: usize,
    pub version: String,
}
#[derive(Debug, Clone)]
pub struct RandomUserAPIClient {
    client: reqwest::Client,
}
impl Default for RandomUserAPIClient {
    fn default() -> Self {
        Self::new("cs25-303-data-tools").unwrap()
    }
}
impl RandomUserAPIClient {
    pub fn new(user_agent: &str) -> Result<Self, RandomUserError> {
        let client = Self {
            client: reqwest::Client::builder().user_agent(user_agent).build()?,
        };
        Ok(client)
    }

    pub async fn fetch_users(&self, count: usize) -> Result<Vec<RandomUser>, RandomUserError> {
        let url = format!(
            "https://randomuser.me/api/?results={}&inc=name,phone,gender&nat=US",
            count
        );
        let response = self.client.get(&url).send().await?;
        let response = response.json::<RandomUserAPIResponse>().await?;
        Ok(response.results)
    }
}
