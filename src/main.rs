use std::{
    collections::HashSet,
    env,
    fs::File,
    io::{Read as _, Write as _},
};

use flate2::bufread::MultiGzDecoder;
use roxmltree::{Document, Node};
use serde::Serialize;

fn walk_target_childs<F>(node: &Node, target_tag: &str, mut callback: F)
where
    F: FnMut(&roxmltree::Node),
{
    for child in node.children() {
        if child.has_tag_name(target_tag) {
            callback(&child);
        }
    }
}

#[derive(Debug, Serialize)]
struct AudioClip {
    start: f64,
    path: String,
}

#[derive(Debug, Serialize)]
struct AlsParseResult {
    paths: Vec<String>,
    audio_clips: Vec<AudioClip>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <als file>", args[0]);
        std::process::exit(1);
    }

    let als_file = File::open(&args[1]).unwrap();
    let mut buf_reader = std::io::BufReader::new(als_file);
    let mut decoded = String::new();

    let mut d = MultiGzDecoder::new(&mut buf_reader);
    d.read_to_string(&mut decoded).unwrap();

    // let output_file = File::create("test.als.xml").unwrap();
    // let mut output = std::io::BufWriter::new(output_file);
    // output.write_all(decoded.as_bytes()).unwrap();

    let als = Document::parse(&decoded).unwrap();
    let als_root = als.root_element();

    let mut audio_clips: Vec<AudioClip> = Vec::new();

    als_root.descendants().for_each(|node| {
        walk_target_childs(&node, "AudioClip", |audio_clip| {
            let mut current_start = String::new();
            let mut path = String::new();

            walk_target_childs(audio_clip, "SampleRef", |sample_ref| {
                walk_target_childs(sample_ref, "FileRef", |file_ref| {
                    walk_target_childs(file_ref, "Path", |path_node| {
                        path = path_node.attribute("Value").unwrap().to_string();
                    });
                });
            });
            walk_target_childs(audio_clip, "CurrentStart", |current_start_node| {
                current_start = current_start_node.attribute("Value").unwrap().to_string();
            });

            audio_clips.push(AudioClip {
                start: current_start.parse().unwrap(),
                path: path,
            })
        });
    });

    audio_clips.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());

    let mut unique_audio_clips: HashSet<String> = HashSet::new();
    for audio_clip in &audio_clips {
        unique_audio_clips.insert(audio_clip.path.clone());
    }

    let als_parse_result = AlsParseResult {
        paths: unique_audio_clips.into_iter().collect(),
        audio_clips: audio_clips,
    };

    let yaml = serde_yaml::to_string(&als_parse_result).unwrap();

    let output_file = File::create("test.als.yaml").unwrap();
    let mut output = std::io::BufWriter::new(output_file);
    output.write_all(yaml.as_bytes()).unwrap();
}
