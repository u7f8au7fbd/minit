use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};

use crate::{
    http::HttpClient,
    models::{InstallChoice, Loader},
};

pub fn install(client: &HttpClient, choice: &InstallChoice, minecraft_dir: &Path) -> Result<()> {
    println!("downloading installer...");
    fs::create_dir_all(minecraft_dir)
        .with_context(|| format!("failed to create {}", minecraft_dir.display()))?;

    let file_name = match choice.loader {
        Loader::Fabric => format!("fabric-installer-{}.jar", env!("CARGO_PKG_VERSION")),
        Loader::NeoForge => format!("neoforge-{}-installer.jar", choice.loader_version),
    };
    let installer = download_to_cache(client, &choice.installer_url, &file_name)?;

    println!(
        "installing {} {} for Minecraft {}...",
        choice.loader.as_str(),
        choice.loader_version,
        choice.minecraft_version
    );

    let mut command = Command::new("java");
    command
        .arg("-jar")
        .arg(&installer)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    match choice.loader {
        Loader::Fabric => {
            command
                .arg("client")
                .arg("-mcversion")
                .arg(&choice.minecraft_version)
                .arg("-loader")
                .arg(&choice.loader_version)
                .arg("-dir")
                .arg(minecraft_dir);
        }
        Loader::NeoForge => {
            command.arg("--install-client").arg(minecraft_dir);
        }
    }

    let status = command.status().context("failed to start Java installer")?;
    if !status.success() {
        bail!("installer exited with {status}");
    }

    Ok(())
}

fn download_to_cache(client: &HttpClient, url: &str, file_name: &str) -> Result<PathBuf> {
    let cache_dir = env::temp_dir().join("minit");
    fs::create_dir_all(&cache_dir)
        .with_context(|| format!("failed to create {}", cache_dir.display()))?;

    let path = cache_dir.join(file_name);
    let bytes = client.bytes(url)?;
    fs::write(&path, bytes).with_context(|| format!("failed to write {}", path.display()))?;

    Ok(path)
}
