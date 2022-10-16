use serde::{Deserialize,Serialize};


pub async fn test_reqwest_serde() -> Result<(), reqwest::Error> {
    let res = reqwest::get("https://raw.githubusercontent.com/NateScarlet/holiday-cn/master/2022.json").await?;
    let data = res.json::<Root>().await?;
    let days = data.days;
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
