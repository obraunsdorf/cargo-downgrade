use chrono::DateTime;
use clap::Parser;
use error_reporter::Report;
use std::{num::NonZeroU8, path::PathBuf};

#[derive(Parser, Debug)]
struct CliArguments {
    /// Path to the Cargo.lock file.
    cargo_lock: Option<PathBuf>,

    /// Date to which the dependencies should be downgraded. In RC 2822 format, e.g. "22 Feb 2021 23:16:09 GMT"
    #[clap(long)]
    date: String,

    /// Dependency level to which transitive dependencies of the crate should be downgraded.
    #[clap(short = 'l', long)]
    dependency_level: Option<NonZeroU8>,
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let args = CliArguments::parse();

    let lock_path = match args.cargo_lock {
        Some(path) => path,
        None => {
            let mut path = std::env::current_dir().unwrap();
            path.push("Cargo.lock");
            path
        }
    };
    let cargo_lock = cargo_lock::Lockfile::load(lock_path).unwrap();
    let dependency_tree = cargo_lock.dependency_tree().unwrap();

    let mut crate_names = downgrade::get_dependencies(args.dependency_level, &dependency_tree);

    // vector has to be sorted for dedup to work
    crate_names.sort();
    crate_names.dedup();

    let datetime: DateTime<chrono::Utc> = DateTime::parse_from_rfc2822(&args.date)
        .unwrap()
        .with_timezone(&chrono::Utc);

    match downgrade::get_downgraded_dependencies(&crate_names, datetime).await {
        Ok(downgraded_dependencies) => {
            for dep in downgraded_dependencies {
                println!("{}", dep);
            }
        }
        Err(err) => {
            eprintln!("Error: {}", Report::new(err));
            std::process::exit(1);
        }
    }
}
