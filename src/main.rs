use mecab::Tagger;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::io::BufRead;
use std::{cell::RefCell, fs::File, io::BufReader};

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct Meta {
    subset: String,
    lang: String,
    id: String,
    author: String,
    userid: u64,
    title: String,
    length: u64,
    points: u64,
    q: f32,
    chapters: u32,
    keywords: Vec<String>,
    isr15: i8,
    genre: Option<u32>,
    biggenre: u32,
    isr18: Option<i8>,
    nocgenre: Option<u64>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Entry {
    text: String,
    #[serde(skip)]
    meta: Meta,
}

fn is_separator(c: char) -> bool {
    c.is_whitespace()
        || c.is_ascii_punctuation()
        || (c == '，')
        || (c == '…')
        || (c == '‥')
        || (c == '。')
        || (c == '！')
        || (c == '？')
}

#[derive(Debug, Clone)]
struct Node {
    text: String,
    pos: String,
    dictionary_form: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct NodeFrequency {
    pos: String,
    dictionary_form: String,
    frequency: u32,
}

fn parse_text(tagger: &mut Tagger, text: &str) -> Vec<Node> {
    let result = tagger.parse_to_node(text);

    result
        .iter_next()
        .filter(|node| {
            !matches!(
                node.stat as i32,
                mecab::MECAB_BOS_NODE | mecab::MECAB_EOS_NODE
            )
        })
        .map(|node| {
            if true {
                let text = node.surface[..node.length as usize].to_string();
                let mut iter = node.feature.split(',');
                let pos = iter.next().unwrap().to_string();
                let mut iter = iter.skip(5);
                let dictionary_form = iter.next().unwrap_or("").to_string();

                Node {
                    text,
                    pos,
                    dictionary_form,
                }
            } else {
                let text = node.surface[..node.length as usize].to_string();
                let mut iter = node.feature.split(',');
                let pos = iter.next().unwrap().to_string();
                let mut iter = iter.skip(8);
                let reading = iter.next().unwrap_or("").to_string();
                let dictionary_form = iter.next().unwrap_or("").to_string();

                Node {
                    text,
                    pos,
                    dictionary_form,
                }
            }
        })
        .collect::<Vec<_>>()
}

fn process_nodes(nodes: Vec<Node>) -> (Vec<Node>, FxHashMap<&'static str, u32>) {
    let mut result = Vec::new();
    let mut inflections_frequency = FxHashMap::default();
    let mut insert_inflection = |inflection: &'static str| {
        inflections_frequency
            .entry(inflection)
            .and_modify(|v| *v += 1)
            .or_insert(1u32);
    };

    let mut i = 0;
    let peek_n = |i: usize, n: usize| {
        if (i + n) < nodes.len() {
            Some(&nodes[i + n])
        } else {
            None
        }
    };
    while i < nodes.len() {
        let node = &nodes[i];

        let a = peek_n(i, 1).map(|v| v.text.as_ref());
        let b = peek_n(i, 2).map(|v| v.text.as_ref());
        let c = peek_n(i, 3).map(|v| v.text.as_ref());
        let d = peek_n(i, 4).map(|v| v.text.as_ref());

        if let "動詞" = node.pos.as_ref() {
            match (a, b, c, d) {
                (Some("ませ"), Some("ん"), Some("でし"), Some("た")) => {
                    insert_inflection("ませんでした");
                    result.push(Node {
                        text: format!("{}ませんでした", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 4;
                }
                (Some("させ"), Some("られ"), Some("ない"), _) => {
                    insert_inflection("させられない");
                    result.push(Node {
                        text: format!("{}させられない", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 3;
                }
                (Some("られ"), Some("ませ"), Some("ん"), _) => {
                    insert_inflection("られません");
                    result.push(Node {
                        text: format!("{}られません", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 3;
                }
                (Some("させ"), Some("ない"), _, _) => {
                    insert_inflection("させない");
                    result.push(Node {
                        text: format!("{}させない", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 2;
                }
                (Some("させ"), Some("られる"), _, _) => {
                    insert_inflection("させられる");
                    result.push(Node {
                        text: format!("{}させられる", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 2;
                }
                (Some("なかっ"), Some("た"), _, _) => {
                    insert_inflection("なかった");
                    result.push(Node {
                        text: format!("{}なかった", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 2;
                }
                (Some("なく"), Some("て"), _, _) => {
                    insert_inflection("なくて");
                    result.push(Node {
                        text: format!("{}なくて", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 2;
                }
                (Some("まし"), Some("た"), _, _) => {
                    insert_inflection("ました");
                    result.push(Node {
                        text: format!("{}ました", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 2;
                }
                (Some("せ"), Some("ない"), _, _) => {
                    insert_inflection("せない");
                    result.push(Node {
                        text: format!("{}せない", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 2;
                }
                (Some("ませ"), Some("ん"), _, _) => {
                    insert_inflection("ません");
                    result.push(Node {
                        text: format!("{}ません", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 2;
                }
                (Some("られ"), Some("ない"), _, _) => {
                    insert_inflection("られない");
                    result.push(Node {
                        text: format!("{}られない", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 2;
                }
                (Some("られ"), Some("ます"), _, _) => {
                    insert_inflection("られます");
                    result.push(Node {
                        text: format!("{}られます", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 2;
                }
                (Some("れ"), Some("ない"), _, _) => {
                    insert_inflection("れない");
                    result.push(Node {
                        text: format!("{}れない", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 2;
                }
                (Some("させる"), _, _, _) => {
                    insert_inflection("させる");
                    result.push(Node {
                        text: format!("{}させる", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 1;
                }
                (Some("せる"), _, _, _) => {
                    insert_inflection("せる");
                    result.push(Node {
                        text: format!("{}せる", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 1;
                }
                (Some("た"), _, _, _) => {
                    insert_inflection("た");
                    result.push(Node {
                        text: format!("{}た", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 1;
                }
                (Some("だ"), _, _, _) => {
                    insert_inflection("だ");
                    result.push(Node {
                        text: format!("{}だ", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 1;
                }
                (Some("て"), _, _, _) => {
                    insert_inflection("て");
                    result.push(Node {
                        text: format!("{}て", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 1;
                }
                (Some("で"), _, _, _) => {
                    insert_inflection("で");
                    result.push(Node {
                        text: format!("{}で", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 1;
                }
                (Some("な"), _, _, _) => {
                    insert_inflection("な");
                    result.push(Node {
                        text: format!("{}な", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 1;
                }
                (Some("ない"), _, _, _) => {
                    insert_inflection("ない");
                    result.push(Node {
                        text: format!("{}ない", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 1;
                }
                (Some("ます"), _, _, _) => {
                    insert_inflection("ます");
                    result.push(Node {
                        text: format!("{}ます", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 1;
                }
                (Some("られる"), _, _, _) => {
                    insert_inflection("られる");
                    result.push(Node {
                        text: format!("{}られる", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 1;
                }
                (Some("れる"), _, _, _) => {
                    insert_inflection("れる");
                    result.push(Node {
                        text: format!("{}れる", node.text),
                        pos: node.pos.clone(),
                        dictionary_form: node.dictionary_form.clone(),
                    });
                    i += 1;
                }
                _ => {
                    result.push(node.clone());
                }
            }
        } else {
            result.push(node.clone());
        }

        i += 1;
    }

    (result, inflections_frequency)
}

fn process_with_ipadic() -> anyhow::Result<()> {
    thread_local! {
        static TAGGER: RefCell<Tagger> = RefCell::new(Tagger::new(""));
    }
    let regex = regex::Regex::new(r"^(\p{Han}|\p{Katakana}|\p{Hiragana})+$")?;
    let spinner_style =
        indicatif::ProgressStyle::with_template("{prefix:.bold.dim}: {elapsed} {spinner} {pos}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    let pb = indicatif::ProgressBar::new(1);
    pb.set_style(spinner_style);

    let mut global_frequency_list = FxHashMap::default();
    let mut global_inflections_frequency = FxHashMap::default();
    for i in 0..=20 {
        pb.set_prefix(format!("[{i:02}/20]"));
        let file = File::open(format!("Syosetu711K/syosetu711k-{i:02}.jsonl"))?;
        let reader = BufReader::new(file);

        let entries = reader
            .lines()
            .map(|line| {
                let entry: Entry = serde_json::from_str(&line.unwrap()).unwrap();
                entry
            })
            .collect::<Vec<_>>();

        let frequency_lists = entries
            .par_iter()
            .map(|entry| {
                pb.inc(1);
                TAGGER.with(|tagger| {
                    let mut frequency_list = FxHashMap::default();
                    let mut inflections_frequency = FxHashMap::default();

                    for part in entry.text.split(is_separator) {
                        let nodes = parse_text(&mut tagger.borrow_mut(), part);
                        let processed_nodes = process_nodes(nodes);

                        for node in processed_nodes.0 {
                            if regex.captures(&node.text).is_some() {
                                frequency_list
                                    .entry(node.text)
                                    .and_modify(|node: &mut NodeFrequency| node.frequency += 1)
                                    .or_insert_with(|| NodeFrequency {
                                        pos: node.pos,
                                        dictionary_form: node.dictionary_form,
                                        frequency: 1,
                                    });
                            }
                        }

                        for (infection, frequency) in processed_nodes.1 {
                            inflections_frequency
                                .entry(infection)
                                .and_modify(|v| *v += frequency)
                                .or_insert(frequency);
                        }
                    }

                    (frequency_list, inflections_frequency)
                })
            })
            .collect::<Vec<_>>();

        for frequency_list in frequency_lists {
            for (key, value) in frequency_list.0 {
                global_frequency_list
                    .entry(key)
                    .and_modify(|node: &mut NodeFrequency| node.frequency += value.frequency)
                    .or_insert_with(|| value);
            }

            for (infection, frequency) in frequency_list.1 {
                global_inflections_frequency
                    .entry(infection)
                    .and_modify(|v| *v += frequency)
                    .or_insert(frequency);
            }
        }
    }

    let output = File::create("frequency_list_ipadic.json")?;
    let writer = std::io::BufWriter::new(output);

    let json = serde_json::json!({
        "verbs": global_frequency_list,
        "inflections": global_inflections_frequency
    });
    serde_json::to_writer(writer, &json)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(32)
        .build_global()
        .unwrap();

    process_with_ipadic()?;

    Ok(())
}
