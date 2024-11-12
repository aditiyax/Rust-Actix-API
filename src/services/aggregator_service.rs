use reqwest::Client;
use serde::Deserialize;


#[derive(Deserialize)]
pub struct AggregatedData {
    pub data1 : String,
    pub data2 : String,
}

pub struct AggregatorService {
    client: Client,
}

impl AggregatorService {
    pub fn new() -> Self {
        Self  {
            client: Client::new(),
        }
    }

    pub async fn fetch_data(&self) -> Result<AggregatedData, reqwest::Error> {
        let response1 = self.client.get("http://api.example.com/data1").send().await?;
        let response2 = self.client.get("http://api.example.com/data2").send().await?;

        let data1: String = response1.text().await?;
        let data2: String = response2.text().await?;

        Ok(AggregatedData {data1, data2})
    }
}