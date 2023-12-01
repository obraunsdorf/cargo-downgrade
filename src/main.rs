use chrono::DateTime;
use clap::{Parser, Subcommand};
use error_reporter::Report;
use std::{num::NonZeroU8, path::PathBuf};

#[derive(Parser, Debug)]
struct CliArguments {
    /// Path to the Cargo.lock file.
    cargo_lock: Option<PathBuf>,

    /// Date to which the dependencies should be downgraded. In RFC 2822 format, e.g. "22 Feb 2021 23:16:09 GMT"
    #[clap(long, short)]
    date: String,

    #[clap(subcommand)]
    modes: DowngradeModes,
}

#[derive(Subcommand, Debug)]
enum DowngradeModes {
    /// Downgrade all crate names of transitive dependencies in Cargo.lock file up to `dependency_level`
    All {
        /// Dependency level to which transitive dependencies of the crate should be downgraded.
        #[clap(long, short = 'l')]
        dependency_level: Option<NonZeroU8>,
    },

    /// Downgrade a list of specific crates
    This {
        /// Comma-separated list of crate names to downgrade
        #[clap(value_delimiter = ',', required = true)]
        crates: Vec<String>,
    },
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

    let crate_names = match &args.modes {
        DowngradeModes::All { dependency_level } => {
            downgrade::get_dependencies(*dependency_level, &dependency_tree)
                .into_iter()
                .collect()
        }
        DowngradeModes::This { crates } => {
            let mut crate_names = crates.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
            // vector has to be sorted for dedup to work
            crate_names.sort();
            crate_names.dedup();
            crate_names
        }
    };

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
