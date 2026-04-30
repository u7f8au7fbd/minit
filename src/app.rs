use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::{
    http::HttpClient,
    models::{InstallChoice, Loader},
    services::{fabric, installer, neoforge},
    tui::{Selection, select_item, select_version},
};

pub struct App {
    client: HttpClient,
    minecraft_dir: PathBuf,
}

enum InstallFlow {
    Selected(InstallChoice),
    Back,
    Quit,
}

impl App {
    pub fn new(client: HttpClient, minecraft_dir: PathBuf) -> Self {
        Self {
            client,
            minecraft_dir,
        }
    }

    pub fn run(&self) -> Result<()> {
        loop {
            let Some(choice) = self.choose_install()? else {
                println!("cancelled");
                return Ok(());
            };

            let summary = vec!["Install".to_string(), "Cancel".to_string()];
            let subtitle = format!(
                "{} / Minecraft {} / Loader {} / Target {}",
                choice.loader.as_str(),
                choice.minecraft_version,
                choice.loader_version,
                self.minecraft_dir.display()
            );

            match select_item("Confirm", &subtitle, &summary)? {
                Selection::Selected(0) => {
                    installer::install(&self.client, &choice, &self.minecraft_dir)?
                }
                Selection::Quit => return Ok(()),
                _ => continue,
            }

            let next = vec!["Install another".to_string(), "Quit".to_string()];
            if select_item("Done", "Launcher profile has been installed.", &next)?
                != Selection::Selected(0)
            {
                break;
            }
        }

        Ok(())
    }

    fn choose_install(&self) -> Result<Option<InstallChoice>> {
        let loaders = vec!["NeoForge".to_string(), "Fabric".to_string()];
        loop {
            let loader_index = match select_item("Mod Loader", "Choose a loader first", &loaders)? {
                Selection::Selected(index) => index,
                Selection::Back | Selection::Quit => return Ok(None),
            };

            let loader = match loader_index {
                0 => Loader::NeoForge,
                1 => Loader::Fabric,
                _ => unreachable!(),
            };

            println!("fetching {} version list...", loader.as_str());

            match loader {
                Loader::Fabric => match self.choose_fabric()? {
                    InstallFlow::Selected(choice) => return Ok(Some(choice)),
                    InstallFlow::Back => continue,
                    InstallFlow::Quit => return Ok(None),
                },
                Loader::NeoForge => match self.choose_neoforge()? {
                    InstallFlow::Selected(choice) => return Ok(Some(choice)),
                    InstallFlow::Back => continue,
                    InstallFlow::Quit => return Ok(None),
                },
            }
        }
    }

    fn choose_fabric(&self) -> Result<InstallFlow> {
        let minecraft_versions = fabric::minecraft_versions(&self.client)?;
        loop {
            let mc_index = match select_version(
                "Minecraft Version",
                "Versions available from Fabric Meta",
                &minecraft_versions,
            )? {
                Selection::Selected(index) => index,
                Selection::Back => return Ok(InstallFlow::Back),
                Selection::Quit => return Ok(InstallFlow::Quit),
            };
            let minecraft_version = minecraft_versions[mc_index].clone();

            println!("fetching Fabric loader versions for Minecraft {minecraft_version}...");

            let loader_versions = fabric::loader_versions(&self.client, &minecraft_version)?;
            let loader_index = match select_version(
                "Fabric Loader",
                &format!("Loader versions for Minecraft {minecraft_version}"),
                &loader_versions,
            )? {
                Selection::Selected(index) => index,
                Selection::Back => continue,
                Selection::Quit => return Ok(InstallFlow::Quit),
            };
            let loader_version = loader_versions[loader_index].clone();
            let installer_url = fabric::installer_url(&self.client)?;

            return Ok(InstallFlow::Selected(InstallChoice {
                loader: Loader::Fabric,
                minecraft_version,
                loader_version,
                installer_url,
            }));
        }
    }

    fn choose_neoforge(&self) -> Result<InstallFlow> {
        let all_versions = neoforge::all_versions(&self.client)?;
        let minecraft_versions = neoforge::minecraft_versions(&all_versions);
        loop {
            let mc_index = match select_version(
                "Minecraft Version",
                "Versions available from NeoForge Maven",
                &minecraft_versions,
            )? {
                Selection::Selected(index) => index,
                Selection::Back => return Ok(InstallFlow::Back),
                Selection::Quit => return Ok(InstallFlow::Quit),
            };
            let minecraft_version = minecraft_versions[mc_index].clone();
            let loader_versions =
                neoforge::loader_versions_for_minecraft(&all_versions, &minecraft_version);

            if loader_versions.is_empty() {
                bail!("no NeoForge versions found for Minecraft {minecraft_version}");
            }

            let loader_index = match select_version(
                "NeoForge Version",
                &format!("Loader versions for Minecraft {minecraft_version}"),
                &loader_versions,
            )? {
                Selection::Selected(index) => index,
                Selection::Back => continue,
                Selection::Quit => return Ok(InstallFlow::Quit),
            };
            let loader_version = loader_versions[loader_index].clone();
            let installer_url = neoforge::installer_url(&loader_version);

            return Ok(InstallFlow::Selected(InstallChoice {
                loader: Loader::NeoForge,
                minecraft_version,
                loader_version,
                installer_url,
            }));
        }
    }
}
