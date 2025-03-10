use clap::{Parser, Subcommand};
use markdown_table::MarkdownTable;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use physis::common::{Language, Platform};
use physis::equipment::{deconstruct_equipment_path, get_slot_from_id};
use physis::exd::ColumnData;
use physis::gamedata::GameData;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    mod_path: String,

    #[clap(short, long, value_parser)]
    game_path: String,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate info for modified equipment.
    GenInfo,
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::GenInfo => {
            let chara_dir_path: PathBuf =
                [args.mod_path, "chara".to_string(), "equipment".to_string()]
                    .iter()
                    .collect();

            let mut game_data = GameData::from_existing(Platform::Win32, &args.game_path).unwrap();

            let items_exh = game_data.read_excel_sheet_header("Item").unwrap();

            let mut item_names: Vec<Vec<String>> = vec![];

            item_names.push(vec!["Item Name".to_string(), "Links".to_string()]);

            for dir in fs::read_dir(chara_dir_path).unwrap() {
                let model_path: PathBuf = [dir.unwrap().path().to_str().unwrap(), "model"]
                    .iter()
                    .collect();

                for model_file in fs::read_dir(model_path).unwrap() {
                    let (model_id, slot) = deconstruct_equipment_path(
                        model_file.unwrap().file_name().to_str().unwrap(),
                    )
                    .unwrap();

                    for page in 0..items_exh.pages.len() {
                        let items_exd = game_data
                            .read_excel_sheet("Item", &items_exh, Language::English, page)
                            .unwrap();

                        for row in &items_exd.rows {
                            let name = if let ColumnData::String(name) = &row.data[9] {
                                name
                            } else {
                                panic!()
                            };
                            let slot_id = if let ColumnData::UInt8(slot_id) = &row.data[17] {
                                slot_id
                            } else {
                                panic!()
                            };

                            if let ColumnData::UInt64(primary_model) = &row.data[47] {
                                let [a, b, c, d, e, f, g, h] = primary_model.to_le_bytes();
                                let number = ((b as u16) << 8) | a as u16;

                                if let Some(islot) = get_slot_from_id(*slot_id as i32) {
                                    if number as i32 == model_id && model_id != 0 && slot == islot {
                                        const FRAGMENT: &AsciiSet = &CONTROLS
                                            .add(b' ')
                                            .add(b'"')
                                            .add(b'<')
                                            .add(b'>')
                                            .add(b'`');
                                        let lodestone_url = format!("<a href='https://na.finalfantasyxiv.com/lodestone/playguide/db/item/?patch=&db_search_category=item&category2=3&category3=&difficulty=&min_item_lv=&max_item_lv=&min_gear_lv=&max_gear_lv=&min_craft_lv=&max_craft_lv=&q={}'>Search Lodestone</a>", utf8_percent_encode(name, FRAGMENT).to_string());

                                        item_names.push(vec![name.clone(), lodestone_url]);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let table = MarkdownTable::new(item_names);

            println!("{:?}", table.to_string());
        }
    }
}
