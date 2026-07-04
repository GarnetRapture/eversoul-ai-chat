use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

pub struct HttpManager {
    client: Client,
    base_url: String,
}

impl HttpManager {
    /// 기본 URL을 설정하여 HTTP 매니저를 생성한다.
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client, base_url }
    }

    /// 서버에 GET 요청을 보내고 JSON 응답을 역직렬화하여 반환한다.
    pub async fn get<T>(&self, path: &str) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP GET 요청 실패: Status {}",
                response.status()
            ));
        }

        let data = response.json::<T>().await?;
        Ok(data)
    }

    /// 서버에 POST 요청(JSON 바디 포함)을 보내고 JSON 응답을 역직렬화하여 반환한다.
    pub async fn post<B, T>(&self, path: &str, body: &B) -> anyhow::Result<T>
    where
        B: Serialize,
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let response = self.client.post(&url).json(body).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP POST 요청 실패: Status {}",
                response.status()
            ));
        }

        let data = response.json::<T>().await?;
        Ok(data)
    }
}
