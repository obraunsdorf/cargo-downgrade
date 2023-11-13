use core::fmt;

use chrono::{DateTime, Utc};
use log::info;
use thiserror::Error;

#[derive(Debug)]
pub struct Package {
    name: String,
    version: String,
    /* source: Option<String>,
    dependencies: Option<HashMap<String, Value>>, */
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = \"{}\"", self.name, self.version)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to read Cargo.lock")]
    ReadCargoLock(#[from] std::io::Error),
    #[error("Failed to parse Cargo.lock")]
    ParseCargoLock(#[from] cargo_lock::Error),
    #[error("Failed to fetch from crates.io")]
    Reqwest(#[from] crates_io_api::Error),
    #[error("No version of crate {0} found before date. Oldest unyanked version is: {1}")]
    VersionNotFound(String, String),
}
type Result<T> = std::result::Result<T, Error>;

/*
/// Get all dependency crate names from in Cargo.lock file that are on the dependency level `dependency_level`
pub fn get_dependencies(dependency_level: Option<u8>, cargo_lock: & cargo_lock::Lockfile) -> Result<Vec<&str>> {
    let tree = cargo_lock.dependency_tree()?;
    let mut crate_names = vec![];

    // initialize the worklist with the root nodes
    let mut worklist = tree.graph().externals(petgraph::Direction::Incoming).collect();

    //TODO
}*/

/// For every defined package in `cargo_lock`, find the version that has been published before `date`
pub async fn get_downgraded_dependencies(
    crate_names: &[&str],
    date: DateTime<Utc>,
) -> Result<Vec<Package>> {
    let cratesio_api_client = crates_io_api::AsyncClient::new(
        "downgrade crawler (https://github.com/obraunsdorf/)", // TODO link to github
        std::time::Duration::from_millis(1000),
    )
    .unwrap();

    // sequentially fetch the version information for all packages since we connect to the crates.io API only every second
    let mut downgraded_dependencies = vec![];
    for crate_name in crate_names {
        info!("fetching infos for crate {}", crate_name);
        let crate_data = cratesio_api_client.get_crate(crate_name).await?;

        // sort versions by release date
        let mut versions = crate_data.versions;
        versions.sort_unstable_by_key(|version| version.updated_at);

        // find the last version that has been published before `date`
        match versions
            .iter()
            .rev()
            .find(|version| version.updated_at < date && !version.yanked)
        {
            Some(version) => {
                downgraded_dependencies.push(Package {
                    version: version.num.clone(),
                    name: (*crate_name).to_owned(),
                });
            }
            None => {
                return Err(Error::VersionNotFound(
                    (*crate_name).to_owned(),
                    versions
                        .iter()
                        .find(|version| !version.yanked)
                        .map(|v| format!("{} ({})", v.num, v.updated_at.format("%Y-%m-%d")))
                        .unwrap_or_else(|| "no known versions at all?".to_owned())
                        .to_owned(),
                ));
            }
        }
    }

    Ok(downgraded_dependencies)
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_get_downgraded_dependencies() {
        let datetime: DateTime<Utc> = DateTime::parse_from_rfc2822("22 Feb 2021 23:16:09 GMT")
            .unwrap()
            .with_timezone(&Utc);
        let crate_names = vec!["serde"];
        let downgraded_dependencies = get_downgraded_dependencies(&crate_names, datetime)
            .await
            .unwrap();
        assert_eq!(downgraded_dependencies[0].version, "1.0.123");
    }
}
