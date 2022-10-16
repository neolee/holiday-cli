use serde::{Deserialize,Serialize};


fn make_data_url(url_prefix: &str, year: i32) -> String {
    String::from(url_prefix) + &(format!("{}.json", year))
}

pub async fn get_holidays_of_year(url_prefix: &str, year: i32) -> Result<Vec<Day>, reqwest::Error> {
    let days = reqwest::get(make_data_url(url_prefix, year))
        .await?
        .json::<Root>()
        .await?
        .days;

    Ok(days)
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    #[serde(rename = "$schema")]
    pub schema: String,
    #[serde(rename = "$id")]
    pub id: String,
    pub year: i64,
    pub papers: Vec<String>,
    pub days: Vec<Day>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Day {
    pub name: String,
    pub date: String,
    pub is_off_day: bool,
}
