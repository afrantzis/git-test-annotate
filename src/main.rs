/*
 * Copyright © 2018 Alexandros Frantzis
 *
 * This program is free software: you can redistribute it and/or modify it
 * under the terms of the GNU General Public License version 3,
 * as published by the Free Software Foundation.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

extern crate git2;
extern crate magic;

use git2::{Repository, Oid, Diff, ObjectType};
use std::path::{Path, PathBuf};
use magic::{Cookie, flags::{MIME_TYPE, SYMLINK}};

fn is_text_file(path: &Path) -> bool {
    let cookie = Cookie::open(MIME_TYPE | SYMLINK).unwrap();
    cookie.load::<&Path>(&[]).unwrap();

    let mut full_path = PathBuf::new();
    full_path.push(std::env::args().nth(1).unwrap());
    full_path.push(path);

    let m = cookie.file(&full_path).unwrap_or("".to_string());
    m.contains("text") && !m.contains("x-po")
}

fn is_test_file(path: &Path) -> bool {
    path.to_str().unwrap().contains("test")
}

fn diff_contains_test_file(diff: Diff) -> bool {
    diff.deltas().any(|d| d.new_file().path().map_or(false, |p| is_test_file(p)))
}

fn commit_contains_test_file(repo: &Repository, commit_id: Oid) -> bool {
    let commit = repo.find_commit(commit_id).unwrap();
    let commit_tree = commit.tree().ok();
    let parent_commit_tree = match commit.parent(0).ok() {
        Some(p) => p.tree().ok(),
        None => None
    };

    let diff = repo.diff_tree_to_tree(
        parent_commit_tree.as_ref(),
        commit_tree.as_ref(), None).unwrap();

    diff_contains_test_file(diff)
}

fn print_size_stats(repo: &Repository) {
    let rev = repo.revparse_single("HEAD").unwrap();

    let mut entries : Vec<_> =
        rev.peel_to_tree().unwrap().iter()
            .map(|e| ("".to_string(), e.to_owned()))
            .collect();

    while entries.len() > 0 {
        let (path, entry) = entries.pop().unwrap();
        let mut full_path = PathBuf::new();
        full_path.push(path);
        full_path.push(entry.name().unwrap());

        match entry.kind() {
            Some(ObjectType::Blob) => {
                let size = entry.
                    to_object(repo).unwrap()
                    .peel_to_blob().unwrap()
                    .content().len();
                if !is_text_file(full_path.as_path()) {
                    println!("[file] {} {} Ignore", full_path.to_str().unwrap(), size);
                } else if is_test_file(full_path.as_path()) {
                    println!("[file] {} {} Test", full_path.to_str().unwrap(), size);
                } else {
                    println!("[file] {} {} NotTest", full_path.to_str().unwrap(), size);
                }
            }
            Some(ObjectType::Tree) => {
                let tree = entry.to_object(repo).unwrap().peel_to_tree().unwrap();
                entries.extend(
                    tree.iter()
                        .map(|e| (
                            full_path.to_str().unwrap().to_string(),
                            e.to_owned()
                        ))
                );
            }
            _ => {}
        };
    }
}

fn print_commit_stats(repo: &Repository) {
    let mut revwalk = repo.revwalk().unwrap();

    revwalk.push_head().unwrap();
    revwalk.simplify_first_parent();

    for commit_id in revwalk {
        let id = commit_id.unwrap();
        if commit_contains_test_file(&repo, id) {
            println!("[commit] {} Test", id);
        } else {
            println!("[commit] {} NotTest", id);
        }
    }
}


fn main() {
    let repo = Repository::open(std::env::args().nth(1).unwrap()).unwrap();

    print_size_stats(&repo);
    print_commit_stats(&repo);
}
