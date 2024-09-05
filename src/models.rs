use std::collections::BTreeMap;

use html_editor::{operation::Htmlifiable, Node};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub aram: BTreeMap<String, AramChampionStats>,
    pub arena: BTreeMap<String, ArenaChampionStats>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AramChampionStats {
    pub name: String,
    pub id: i32,
    pub damage_dealt: f32,
    pub damage_received: f32,
    pub other: String,
}

impl AramChampionStats {
    pub fn new(value: &Vec<Node>) -> Self {
        Self {
            name: html_escape::decode_html_entities(
                &value[0]
                    .as_element()
                    .unwrap()
                    .attrs
                    .clone()
                    .into_iter()
                    .find(|p| p.0 == "data-sort-value")
                    .unwrap()
                    .1,
            )
            .to_string(),
            id: 0,
            damage_dealt: parse_damage_stat(value[1].html()),
            damage_received: parse_damage_stat(value[2].html()),
            other: get_inner_html(&value[3]),
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ArenaChampionStats {
    pub name: String,
    pub id: i32,
    pub other: String,
}

impl ArenaChampionStats {
    pub fn new(value: &Vec<Node>) -> Self {
        Self {
            name: html_escape::decode_html_entities(
                &value[0]
                    .as_element()
                    .unwrap()
                    .attrs
                    .clone()
                    .into_iter()
                    .find(|p| p.0 == "data-sort-value")
                    .unwrap()
                    .1,
            )
            .to_string(),
            id: 0,
            other: get_inner_html(&value[3]),
        }
    }
}

fn parse_damage_stat(html: String) -> f32 {
    let text = strip_html(html);
    if text.len() == 0 {
        return 100.0;
    }

    return 100.0 + text.replace("%", "").parse::<f32>().unwrap();
}

fn strip_html(html: String) -> String {
    let mut data = String::new();
    let mut inside = false;

    for c in html.chars() {
        if c == '<' {
            inside = true;
            continue;
        }
        if c == '>' {
            inside = false;
            continue;
        }
        if !inside {
            data.push(c);
        }
    }

    data
}

fn get_inner_html(value: &Node) -> String {
    let trim = {
        let mut data = String::new();
        let mut pass = false;

        for c in value.html().chars() {
            if !pass && c == '>' {
                pass = true;
                continue;
            }
            if pass {
                data.push(c);
            }
        }

        data
    };

    {
        let mut data = String::new();
        let mut pass = false;

        for c in trim.chars().rev() {
            if !pass && c == '<' {
                pass = true;
                continue;
            }
            if pass {
                data.push(c);
            }
        }

        data.chars().rev().collect::<String>().trim().to_string()
    }
}

#[derive(Deserialize)]
pub struct ChampionSummary {
    pub name: String,
    pub id: i32,
    pub alias: String,
}
