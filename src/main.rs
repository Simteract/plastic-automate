use clap::{App, Arg, SubCommand};
use serde::{Deserialize, Serialize};
use std::{env::current_dir, process::Command};

#[derive(Debug, Default, Serialize, Deserialize)]
struct StatusOutput {
    #[serde(rename = "Changes", default)]
    pub changes: Changes,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Changes {
    #[serde(rename = "Change", default)]
    pub changes: Vec<Change>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Change {
    #[serde(rename = "Path", default)]
    pub path: String,
    #[serde(rename = "PrintableSize", default)]
    pub size: String,
}

impl ToString for Change {
    fn to_string(&self) -> String {
        format!("File `{}` of size: {}", self.path, self.size)
    }
}

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Automate Plastic SCM")
        .subcommand(
            SubCommand::with_name("ensure")
                .about("Ensure repository has no pending changes (undo if any)")
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .short("v")
                        .required(false)
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("log")
                        .long("log")
                        .short("l")
                        .required(false)
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("working-dir")
                        .long("working-dir")
                        .short("w")
                        .required(false)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("update")
                .about("Update to latest and redownload corrupted files")
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .short("v")
                        .required(false)
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("log")
                        .long("log")
                        .short("l")
                        .required(false)
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("working-dir")
                        .long("working-dir")
                        .short("w")
                        .required(false)
                        .takes_value(true),
                ),
        )
        .get_matches();
    match matches.subcommand() {
        ("ensure", Some(matches)) => {
            let working_dir = match matches.value_of("working-dir") {
                Some(value) => value.to_owned(),
                None => current_dir()
                    .expect("Cannot get current directory")
                    .to_string_lossy()
                    .into_owned(),
            };
            let verbose = matches.is_present("verbose");
            let log = matches.is_present("log");
            ensure_clean(&working_dir, verbose, log);
            if verbose {
                println!("* Done!");
            }
        }
        ("update", Some(matches)) => {
            let working_dir = match matches.value_of("working-dir") {
                Some(value) => value.to_owned(),
                None => current_dir()
                    .expect("Cannot get current directory")
                    .to_string_lossy()
                    .into_owned(),
            };
            let verbose = matches.is_present("verbose");
            let log = matches.is_present("log");
            update(&working_dir, verbose, log);
            if verbose {
                println!("* Done!");
            }
        }
        _ => {}
    }
}

fn update(working_dir: &str, verbose: bool, log: bool) {
    ensure_clean(working_dir, verbose, log);
    update_latest(working_dir, verbose, log);
    ensure_clean(working_dir, verbose, log);
}

fn ensure_clean(working_dir: &str, verbose: bool, log: bool) {
    if verbose {
        println!("* Ensure clean workspace");
    }
    loop {
        let status = get_status(working_dir, verbose, log);
        if status.changes.changes.is_empty() {
            break;
        }
        if verbose {
            println!("* Workspace has pending changes:");
            for change in &status.changes.changes {
                println!("- {}", change.to_string());
            }
        }
        cleanup(&status.changes.changes, working_dir, verbose, log);
    }
}

fn get_status(working_dir: &str, verbose: bool, log: bool) -> StatusOutput {
    if verbose {
        println!("* Get workspace status");
    }
    let output = Command::new("cm")
        .arg("status")
        .arg("--xml")
        .arg("--fullpath")
        .current_dir(working_dir)
        .output()
        .expect("Error during `cm status`");
    if log {
        let contents = String::from_utf8_lossy(&output.stdout);
        println!("* STDOUT: `{}`", contents);
        let contents = String::from_utf8_lossy(&output.stderr);
        println!("* STDERR: `{}`", contents);
    }
    let contents = String::from_utf8_lossy(&output.stdout);
    serde_xml_rs::from_str::<StatusOutput>(&contents)
        .expect(&format!("Cannot deserialize `{}`", contents))
}

fn cleanup(changes: &[Change], working_dir: &str, verbose: bool, log: bool) {
    if verbose {
        println!("* Undo changes");
    }
    for change in changes {
        undo(change, working_dir, verbose, log);
    }
}

fn undo(change: &Change, working_dir: &str, verbose: bool, log: bool) {
    if verbose {
        println!("* Undo change: {}", change.to_string());
    }
    let output = Command::new("cm")
        .arg("undo")
        .arg(&change.path)
        .current_dir(working_dir)
        .output()
        .expect("Error during `cm undo`");
    if log {
        let contents = String::from_utf8_lossy(&output.stdout);
        println!("* STDOUT: `{}`", contents);
        let contents = String::from_utf8_lossy(&output.stderr);
        println!("* STDERR: `{}`", contents);
    }
}

fn update_latest(working_dir: &str, verbose: bool, log: bool) {
    if verbose {
        println!("* Update workspace");
    }
    let output = Command::new("cm")
        .arg("update")
        .arg("--last")
        .arg("--override")
        .arg("--forced")
        .current_dir(working_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .expect("Error during `cm update`");
    if log {
        let contents = String::from_utf8_lossy(&output.stdout);
        println!("* STDOUT: `{}`", contents);
        let contents = String::from_utf8_lossy(&output.stderr);
        println!("* STDERR: `{}`", contents);
    }
}
