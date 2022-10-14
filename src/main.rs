use serde::{Deserialize,Serialize};
// use reqwest::blocking::get;


fn main() {
    println!("Hello, world!");
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
