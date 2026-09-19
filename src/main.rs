mod image_hashing;
mod cli_args;
mod file_utils;

use crate::cli_args::CliArgs;
use crate::file_utils::get_images_from_target_directory;
use crate::image_hashing::HashedImageEntry;
use clap::Parser;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use petgraph::graph::UnGraph;
use petgraph::prelude::EdgeRef;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use vp_tree::{Querry, VpTree};

fn main() {
    let cli = CliArgs::parse();
    if let Err(e) = cli.validate() {
        println!("Invalid arguments, {}", e);
        return;
    }

    let dir_scanning_progress_bar = ProgressBar::new_spinner();
    dir_scanning_progress_bar.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {spinner:.green} Files Scanned: {pos} | Images Found: {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏⠿")
    );
    dir_scanning_progress_bar.enable_steady_tick(Duration::from_millis(100));
    let files = match get_images_from_target_directory(&cli.target, Some(&dir_scanning_progress_bar)) {
        Ok(files) => files,
        Err(e) => {
            println!("{e}");
            return;
        }
    };
    dir_scanning_progress_bar.finish();

    let image_hashes = match read_and_hash_files(&files) {
        Ok(hashes) => hashes,
        Err(e) => panic!("Failed to process images: {}", e),
    };
    println!("Done!");

    println!("Creating VP-Tree...");
    let vptree = VpTree::new(image_hashes);
    println!("Done");

    println!("Deduplicating Graph...");
    let deduplicated = get_similar_groupings(vptree, cli.distance);
    println!("Done! Have {} images after deduplication", deduplicated.len());

    match write_groupings_to_output(deduplicated, &cli.output) {
        Ok(_) => println!("Done!"),
        Err(e) => println!("Failed to write results, {e}"),
    }
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

    hashed_image_entries
}

fn get_similar_groupings(tree: VpTree<HashedImageEntry>, distance_threshold: i32) -> Vec<Vec<HashedImageEntry>> {
    let mut graph = UnGraph::<usize, ()>::new_undirected();

    let mut node_map = HashMap::new();
    for entry in tree.items() {
        let node_index = graph.add_node(entry.id);
        node_map.insert(entry.id, node_index);
    }

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
        groups.entry(root).or_default().push(item.clone());
    }

    groups.values().cloned().collect()
}

fn write_groupings_to_output(groupings: Vec<Vec<HashedImageEntry>>, output_path: &PathBuf) -> Result<(), String> {
    if !output_path.exists() {
        match fs::create_dir_all(output_path) {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to create output directory: {}", e)),
        }
    }

    let total_images = groupings.iter()
        .map(|grouping| grouping.len() as u64)
        .sum();
    let pb = ProgressBar::new(total_images);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] Writing Files [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) | ETA: {eta}"
            )
            .unwrap()
            .progress_chars("#>-")
    );

    for (i, grouping) in groupings.iter().enumerate() {
        if grouping.len() == 1 {
            let entry = grouping.first().unwrap();
            let filename = entry.path.file_name().unwrap().to_str().unwrap_or("");
            let destination = output_path.join(filename);
            fs::copy(&entry.path, &destination).map_err(|e| format!("Failed to copy: {}", e))?;
            pb.inc(1);
            continue;
        }

        let grouping_dir = output_path.join(i.to_string());
        match fs::create_dir_all(&grouping_dir) {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to create output directory: {}", e)),
        }
        for entry in grouping {
            let filename = entry.path.file_name().unwrap().to_str().unwrap_or("");
            let destination = grouping_dir.join(filename);
            fs::copy(&entry.path, &destination).map_err(|e| format!("Failed to copy: {}", e))?;
            pb.inc(1);
        }
    }

    pb.finish();
    Ok(())
}