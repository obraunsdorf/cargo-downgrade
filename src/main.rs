use chrono::DateTime;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct CliArguments {
    /// Path to the Cargo.lock file
    #[structopt(short, long)]
    cargo_lock: String,

    /// Date string
    #[structopt(short, long)]
    date: String,
}

#[tokio::main]
async fn main() {
    let opt = CliArguments::from_args();

    simple_logger::init_with_level(log::Level::Info).unwrap();

    let cargo_lock = cargo_lock::Lockfile::load(opt.cargo_lock).unwrap();

    let crate_names: Vec<&str> = cargo_lock
        .packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();

    let datetime: DateTime<chrono::Utc> = DateTime::parse_from_rfc2822(&opt.date)
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
