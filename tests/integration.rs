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
extern crate tempdir;

use std::process::{Command, Output};
use std::path::Path;
use tempdir::TempDir;

fn run_test_stats(path: &str) -> Output {
	Command::new("cargo")
		.arg("run")
		.arg("--release")
		.arg("--quiet")
		.arg("--")
		.arg(path)
		.output().unwrap()
}

fn run_cmd(path: &Path, cmd: &str) {
	let split: Vec<&str> = 
		if cmd.starts_with("sh -c '") {
			vec!["sh", "-c", &cmd[7..cmd.len()-1]]
		} else {
			cmd.split(' ').collect()
		};
			
	Command::new(split[0])
		.args(&split[1..])
		.current_dir(path)
		.env("HOME", path)
		.env("GIT_CONFIG_NOSYSTEM", "1")
		.env("GIT_AUTHOR_NAME", "Name")
		.env("GIT_AUTHOR_EMAIL", "email@email.test")
		.env("GIT_AUTHOR_DATE", "2015-10-21T16:29-07:00")
		.env("GIT_COMMITTER_NAME", "Name")
		.env("GIT_COMMITTER_EMAIL", "email@email.test")
		.env("GIT_COMMITTER_DATE", "2015-10-21T16:29-07:00")
		.spawn().unwrap().wait().unwrap();

}

fn run_cmds(path: &Path, cmds: &[&str]) {
	for cmd in cmds {
		run_cmd(path, cmd);
	}
}

fn create_repo(cmds: &[&str]) -> TempDir {
	let tmpdir = TempDir::new("test-stats").unwrap();
	run_cmd(tmpdir.path(), "git init .");
	run_cmds(tmpdir.path(), cmds);
	tmpdir
}

fn find_commits(out: &str) -> Vec<&str> {
	out.lines().filter(|s| s.starts_with("[commit]")).collect()
}

#[test]
fn lists_files() {
	let repodir = create_repo(&vec![
		"sh -c '/bin/echo -n a2 > a2.txt'",
		"sh -c '/bin/echo -n a33 > a33.txt'",
		"sh -c '/bin/echo -n test > test1.txt'",
		"git add .",
		"git commit -a -m 'a'"]);

	let ret = run_test_stats(repodir.path().to_str().unwrap());
	assert!(ret.status.success());

	let out = std::str::from_utf8(&ret.stdout).unwrap();

	assert!(out.contains("[file] a2.txt 2 NotTest"));
	assert!(out.contains("[file] a33.txt 3 NotTest"));
	assert!(out.contains("[file] test1.txt 4 Test"));
}

#[test]
fn ignores_non_text_files() {
	let repodir = create_repo(&vec![
		"sh -c '/bin/echo -n a2 > a2.txt'",
		"sh -c '/bin/echo -n -e \"\\x80PNG\\x0d\\x0a\\x1a\\x0a\" > test1.png'",
		"sh -c '/bin/echo -n -e \"\\x80PNG\\x0d\\x0a\\x1a\\x0a\" > bla.png'",
		"git add .",
		"git commit -a -m 'a'"]);

	let ret = run_test_stats(repodir.path().to_str().unwrap());
	assert!(ret.status.success());

	let out = std::str::from_utf8(&ret.stdout).unwrap();

	assert!(out.contains("[file] a2.txt 2 NotTest"));
	assert!(out.contains("[file] test1.png 8 Ignore"));
	assert!(out.contains("[file] bla.png 8 Ignore"));
}

#[test]
fn ignores_po_files() {
	let repodir = create_repo(&vec![
		"sh -c '/bin/echo -n a2 > a2.txt'",
		r#"sh -c '/bin/echo -n -e "\nmsgid=\"\"\nmsgstr=\"\"" > bla.po'"#,
		"git add .",
		"git commit -a -m 'a'"]);

	let ret = run_test_stats(repodir.path().to_str().unwrap());
	assert!(ret.status.success());

	let out = std::str::from_utf8(&ret.stdout).unwrap();

	assert!(out.contains("[file] a2.txt 2 NotTest"));
	assert!(out.contains("[file] bla.po 19 Ignore"));
}

#[test]
fn lists_commits() {
	let repodir = create_repo(&vec![
		"sh -c '/bin/echo -n a2 > a.txt'",
		"git add .",
		"git commit -a -m 'a'",

		"sh -c '/bin/echo -n b2 > test.txt'",
		"git add .",
		"git commit -a -m 'b'",

		"sh -c '/bin/echo -n more >> test.txt'",
		"sh -c '/bin/echo -n c2 > c.txt'",
		"git add .",
		"git commit -a -m 'mixed'"
	]);

	let ret = run_test_stats(repodir.path().to_str().unwrap());
	assert!(ret.status.success());

	let out = std::str::from_utf8(&ret.stdout).unwrap();

	let commits = find_commits(out);
	assert!(commits.len() == 3);
	assert!(commits[0] == "[commit] fe9ec6b915a59d4adc20fe4518a0ddefd7295668 Test");
	assert!(commits[1] == "[commit] 1ee809f9f2587617a3ea675ab6a52407f1a49ff8 Test");
	assert!(commits[2] == "[commit] 3795230eec6068d3be7b5092c04300f0b2aeb60d NotTest");
}

#[test]
fn ignores_non_mainline_commits() {
	let repodir = create_repo(&vec![
		"sh -c '/bin/echo -n a2 > a.txt'",
		"git add .",
		"git commit -a -m 'a'",
		
		"sh -c '/bin/echo -n b2 > b.txt'",
		"git add .",
		"git commit -a -m 'b'",

		"git checkout -b feature",
		"sh -c '/bin/echo -n c2 > c.txt'",
		"git add .",
		"git commit -a -m 'c'",
		"sh -c '/bin/echo -n d2 > test1.txt'",
		"git add .",
		"git commit -a -m 'd'",

		"git checkout master",
		"git merge -m 'merge' --no-ff feature"]);

	let ret = run_test_stats(repodir.path().to_str().unwrap());
	assert!(ret.status.success());

	let out = std::str::from_utf8(&ret.stdout).unwrap();

	let commits = find_commits(out);
	println!("{}", out);
	assert!(commits.len() == 3);
	assert!(commits[0] == "[commit] cd86be20b74488251027c2cbbd1fab805e34e745 Test");
	assert!(commits[1] == "[commit] 0f715be3cf336775fa10d8e11528b21470050bb0 NotTest");
	assert!(commits[2] == "[commit] 3795230eec6068d3be7b5092c04300f0b2aeb60d NotTest");
}
