mod image_hashing;

use crate::image_hashing::HashedImageEntry;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use petgraph::graph::UnGraph;
use petgraph::prelude::EdgeRef;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::{env, fs};
use vp_tree::{Querry, VpTree};

fn main() {
    println!("Finding files in current directory...");

    let curr_path = match env::current_dir() {
        Ok(path) => path,
        Err(e) => panic!("Failed to read current directory: {}", e),
    };
    let files = get_files_from_path(curr_path);

    println!("Found {} valid image files", files.len());

    println!("Computing hashes...");
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

fn get_files_from_path(path: PathBuf) -> Vec<PathBuf> {
    let dir_entries = match fs::read_dir(&path) {
        Ok(dir_entries) => dir_entries,
        Err(error) => panic!("failed to read directory: {}", error),
    };

    let mut files: Vec<PathBuf> = Vec::new();

    for entry in dir_entries {
        let path = entry.unwrap().path();

        if path.is_dir() {
            files.append(&mut get_files_from_path(path));
        } else if path.is_file() && is_valid_extension(path.extension()) {
            files.push(path);
        }
    }

    return files;
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
            "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) | ETA: {eta}"
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