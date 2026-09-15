mod image_hashing;
mod cli_args;

use crate::cli_args::CliArgs;
use crate::image_hashing::HashedImageEntry;
use clap::Parser;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use petgraph::graph::UnGraph;
use petgraph::prelude::EdgeRef;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::Duration;
use vp_tree::{Querry, VpTree};
use walkdir::WalkDir;

fn main() {
    let cli = CliArgs::parse();
    if let Err(e) = cli.validate() {
        println!("Invalid arguments, {}", e);
        return;
    }

    println!("Finding files in current directory...");
    let files = match scan_target_directory(cli.target) {
        Ok(files) => files,
        Err(e) => {
            println!("{e}");
            return;
        }
    };

    let image_hashes = match read_and_hash_files(&files) {
        Ok(hashes) => hashes,
        Err(e) => panic!("Failed to process images: {}", e),
    };
    println!("Done!");

    println!("Creating VP-Tree...");
    let vptree = VpTree::new(image_hashes);
    println!("Done");

    println!("Deduplicating Graph...");
    let deduplicated = deduplicate(vptree);
    println!("Done! Have {} images after deduplication", deduplicated.len());
}

fn scan_target_directory(target: PathBuf) -> Result<Vec<PathBuf>, String> {
    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {spinner:.green} Files Scanned: {pos} | Images Found: {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
    );
    progress_bar.enable_steady_tick(Duration::from_millis(100));
    let mut supported_images = Vec::new();

    for entry_result in WalkDir::new(target).into_iter() {
        let entry = entry_result.map_err(|e| format!("Failed to process entry: {}", e))?;

        if !entry.file_type().is_file() {
            continue;
        }

        progress_bar.inc(1);
        if is_valid_extension(entry.path().extension()) {
            supported_images.push(entry.path().to_path_buf());
            progress_bar.set_message(supported_images.len().to_string());
        }
    }

    progress_bar.finish();
    Ok(supported_images)
}

fn is_valid_extension(extension: Option<&OsStr>) -> bool {
    let valid_extensions = vec!["jpg", "jpeg", "png", "heic"];

    if let Some(e) = extension {
        return valid_extensions.contains(&e.to_ascii_lowercase().to_str().unwrap());
    }

    return false;
}

fn read_and_hash_files(file_paths: &Vec<PathBuf>) -> Result<Vec<HashedImageEntry>, String> {
    let total_files = file_paths.len() as u64;
    let pb = ProgressBar::new(total_files);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] Computing Hashes [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) | ETA: {eta}"
            )
            .unwrap()
            .progress_chars("#>-")
    );

    let hashed_image_entries: Result<Vec<HashedImageEntry>, String> = file_paths
        .par_iter()
        .enumerate()
        .progress_with(pb)
        .map(|(id, file_path)| HashedImageEntry::create_from_path(id, file_path))
        .collect();

    Ok(hashed_image_entries?)
}

fn deduplicate(tree: VpTree<HashedImageEntry>) -> Vec<HashedImageEntry> {
    let mut graph = UnGraph::<usize, ()>::new_undirected();

    let mut node_map = HashMap::new();
    for entry in tree.items() {
        let node_index = graph.add_node(entry.id);
        node_map.insert(entry.id, node_index);
    }

    let distance_threshold = 5;
    for entry in tree.items() {
        let matches = tree.querry(entry, Querry::new(99999, distance_threshold.into(), true, false));
        for matched_entry in matches {
            let node_a_idx = node_map[&entry.id];
            let node_b_idx = node_map[&matched_entry.id];
            graph.add_edge(node_a_idx, node_b_idx, ());
        }
    }

    let mut vertex_sets = petgraph::unionfind::UnionFind::new(tree.items().len());
    for edge in graph.edge_references() {
        let u = edge.source();
        let v = edge.target();

        vertex_sets.union(u.index(), v.index());
    }

    let mut groups: HashMap<usize, Vec<HashedImageEntry>> = HashMap::new();
    for item in tree.items() {
        let node_idx = node_map[&item.id].index();
        let root = vertex_sets.find(node_idx);
        groups.entry(root).or_insert_with(|| Vec::new()).push(item.clone());
    }

    let mut unique_entries: Vec<HashedImageEntry> = Vec::new();
    for (_, cluster) in &groups {
        if let Some(first) = cluster.first() {
            unique_entries.push(first.clone());
        }
    }

    return unique_entries;
}