use anyhow::{Result, anyhow, bail};
use serde::Deserialize;

use crate::{http::HttpClient, version};

const GAMES_URL: &str = "https://meta.fabricmc.net/v2/versions/game";
const INSTALLERS_URL: &str = "https://meta.fabricmc.net/v2/versions/installer";

#[derive(Debug, Deserialize)]
struct GameVersion {
    version: String,
    stable: bool,
}

#[derive(Debug, Deserialize)]
struct LoaderEntry {
    loader: LoaderVersion,
}

#[derive(Debug, Deserialize)]
struct LoaderVersion {
    version: String,
    stable: bool,
}

#[derive(Debug, Deserialize)]
struct InstallerVersion {
    url: String,
    stable: bool,
}

pub fn minecraft_versions(client: &HttpClient) -> Result<Vec<String>> {
    let versions = client.json::<Vec<GameVersion>>(GAMES_URL)?;

    let mut releases = versions
        .into_iter()
        .filter(|version| version.stable)
        .map(|version| version.version)
        .collect::<Vec<_>>();
    version::sort_desc(&mut releases);

    if releases.is_empty() {
        bail!("Fabric returned no stable Minecraft versions");
    }

    Ok(releases)
}

pub fn loader_versions(client: &HttpClient, minecraft_version: &str) -> Result<Vec<String>> {
    let url = format!("https://meta.fabricmc.net/v2/versions/loader/{minecraft_version}");
    let versions = client.json::<Vec<LoaderEntry>>(&url)?;

    let mut stable = Vec::new();
    let mut unstable = Vec::new();
    for entry in versions {
        if entry.loader.stable {
            stable.push(entry.loader.version);
        } else {
            unstable.push(entry.loader.version);
        }
    }
    stable.extend(unstable);
    version::sort_desc(&mut stable);

    if stable.is_empty() {
        bail!("Fabric returned no loader versions for Minecraft {minecraft_version}");
    }

    Ok(stable)
}

pub fn installer_url(client: &HttpClient) -> Result<String> {
    let installers = client.json::<Vec<InstallerVersion>>(INSTALLERS_URL)?;

    installers
        .iter()
        .find(|installer| installer.stable)
        .or_else(|| installers.first())
        .map(|installer| installer.url.clone())
        .ok_or_else(|| anyhow!("Fabric returned no installer versions"))
}
