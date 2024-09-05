use std::{collections::BTreeMap, error::Error, fs};

use clap::Parser;
use html_editor::{
    operation::{Queryable, Selector},
    Node,
};
use models::{AramChampionStats, ArenaChampionStats, ChampionSummary, Stats};
use rmp_serde::Serializer;
use serde::Serialize;

pub mod models;

#[derive(clap::Parser)]
#[clap(version)]
struct Args {
    // The data format to use for the generated data
    #[clap(short, long, default_value_t, value_enum)]
    format: Format,
}

#[derive(clap::ValueEnum, Clone, Default)]
enum Format {
    // JSON, lightweight data-interchange format
    #[default]
    Json,
    // MessagePack, efficient binary serialization format
    MessagePack,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let summaries = {
        let json = ureq::get("https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-summary.json")
            .call()?
            .into_string()?;
        let vec: Vec<ChampionSummary> = serde_json::from_str(&json)?;

        vec
    };

    let aram_champion_stats = {
        let html = ureq::get("https://leagueoflegends.fandom.com/wiki/ARAM")
            .call()?
            .into_string()?;

        html_editor::parse(&html)?
            .query(&Selector::from("div.tabber"))
            .unwrap()
            .query(&Selector::from("table"))
            .unwrap()
            .query(&Selector::from("tbody"))
            .unwrap()
            .query_all(&Selector::from("tr"))
            .into_iter()
            .skip(1)
            .map(|x| {
                let mut stats = AramChampionStats::new(&x.children);
                let summary = summaries
                    .iter()
                    .find(|p| p.name == stats.name && !p.alias.contains("Strawberry"))
                    .unwrap();
                stats.id = summary.id;
                (summary.alias.clone(), stats)
            })
            .collect::<BTreeMap<String, AramChampionStats>>()
    };

    let arena_champion_stats = {
        let html = ureq::get("https://leagueoflegends.fandom.com/wiki/Arena_(League_of_Legends)")
            .call()?
            .into_string()?;

        html_editor::parse(&html)?
            .query_all(&Selector::from("div.tabber"))
            .into_iter()
            .map(|x| x.clone().into_node())
            .collect::<Vec<Node>>()
            .query_all(&Selector::from("table.article-table.sortable"))
            .into_iter()
            .map(|x| x.clone().into_node())
            .collect::<Vec<Node>>()
            .query(&Selector::from("tbody"))
            .unwrap()
            .query_all(&Selector::from("tr"))
            .into_iter()
            .skip(1)
            .map(|x| {
                let mut stats = ArenaChampionStats::new(&x.children);
                let summary = summaries
                    .iter()
                    .find(|p| p.name == stats.name && !p.alias.contains("Strawberry"))
                    .unwrap();
                stats.id = summary.id;
                (summary.alias.clone(), stats)
            })
            .collect::<BTreeMap<String, ArenaChampionStats>>()
    };

    let stats = Stats {
        aram: aram_champion_stats,
        arena: arena_champion_stats,
    };

    match args.format {
        Format::Json => fs::write("stats.json", serde_json::to_string(&stats).unwrap())?,
        Format::MessagePack => {
            let mut buf = Vec::new();
            let mut serializer =
                Serializer::new(&mut buf).with_bytes(rmp_serde::config::BytesMode::ForceAll);
            stats.serialize(&mut serializer)?;
            fs::write("stats.msgpack", buf)?;
        }
    };

    Ok(())
}
