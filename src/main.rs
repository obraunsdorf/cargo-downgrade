use chrono::DateTime;
use clap::Parser;
use std::{num::NonZeroU8, path::PathBuf};

#[derive(Parser, Debug)]
struct CliArguments {
    /// Path to the Cargo.lock file
    cargo_lock: PathBuf,

    /// Date string
    #[clap(long)]
    date: String,

    /// Dependency level
    #[clap(short, long)]
    dependency_level: Option<NonZeroU8>,
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let args = CliArguments::parse();

    let cargo_lock = cargo_lock::Lockfile::load(args.cargo_lock).unwrap();
    let dependency_tree = cargo_lock.dependency_tree().unwrap();

    let crate_names = downgrade::get_dependencies(args.dependency_level, &dependency_tree);
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
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}
