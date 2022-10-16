use serde::{Deserialize,Serialize};


fn make_data_url(prefix: &str, year: i32) -> String {
    String::from(prefix) + &(format!("{}.json", year))
}

async fn get_holidays_by_year(prefix: &str, year: i32) -> Result<Vec<Day>, reqwest::Error> {
    let days = reqwest::get(make_data_url(prefix, year))
        .await?
        .json::<Root>()
        .await?
        .days;

    Ok(days)
}

pub async fn test_reqwest_serde(prefix: &str) -> Result<(), reqwest::Error> {
    let year = 2020;
    let days = get_holidays_by_year(prefix, year).await?;

    for day in days {
        println!("name: {} date: {} is_off_day: {}", day.name, day.date, day.is_off_day);
    }

    Ok(())
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
