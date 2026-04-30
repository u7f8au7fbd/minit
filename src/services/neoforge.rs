use std::collections::BTreeSet;

use anyhow::{Context, Result, bail};
use quick_xml::{Reader, events::Event as XmlEvent};

use crate::{http::HttpClient, version};

const METADATA_URL: &str =
    "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";

pub fn all_versions(client: &HttpClient) -> Result<Vec<String>> {
    let metadata = client.text(METADATA_URL)?;

    let versions = extract_xml_values(&metadata, "version")?;
    if versions.is_empty() {
        bail!("NeoForge metadata contained no versions");
    }

    let mut versions = versions;
    version::sort_desc(&mut versions);
    Ok(versions)
}

pub fn minecraft_versions(versions: &[String]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut minecraft_versions = Vec::new();

    for version in versions {
        if let Some(minecraft_version) = minecraft_version_for_loader(version) {
            if seen.insert(minecraft_version.clone()) {
                minecraft_versions.push(minecraft_version);
            }
        }
    }

    version::sort_desc(&mut minecraft_versions);

    minecraft_versions
}

pub fn loader_versions_for_minecraft(versions: &[String], minecraft_version: &str) -> Vec<String> {
    let mut loader_versions = versions
        .iter()
        .filter(|version| version_matches_minecraft(version, minecraft_version))
        .cloned()
        .collect::<Vec<_>>();

    version::sort_desc(&mut loader_versions);
    loader_versions
}

pub fn installer_url(version: &str) -> String {
    format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{version}/neoforge-{version}-installer.jar"
    )
}

fn minecraft_version_for_loader(version: &str) -> Option<String> {
    let numeric = version
        .split(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
        .next()?;
    let parts = numeric
        .split('.')
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let major = *parts.first()?;

    if !(20..=99).contains(&major) {
        return None;
    }

    if major >= 26 {
        let minor = parts.get(1)?;
        let patch = parts.get(2)?;
        Some(format!("{major}.{minor}.{patch}"))
    } else {
        let minor = parts.get(1)?;
        if *minor == 0 {
            Some(format!("1.{major}"))
        } else {
            Some(format!("1.{major}.{minor}"))
        }
    }
}

fn version_matches_minecraft(version: &str, minecraft_version: &str) -> bool {
    if minecraft_version_for_loader(version).as_deref() == Some(minecraft_version) {
        true
    } else if let Some(stripped) = minecraft_version.strip_prefix("1.") {
        version.starts_with(stripped)
    } else {
        version.starts_with(minecraft_version)
    }
}

fn extract_xml_values(xml: &str, tag: &str) -> Result<Vec<String>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut in_tag = false;
    let mut values = Vec::new();

    loop {
        match reader.read_event() {
            Ok(XmlEvent::Start(element)) if element.name().as_ref() == tag.as_bytes() => {
                in_tag = true;
            }
            Ok(XmlEvent::Text(text)) if in_tag => {
                values.push(text.decode()?.into_owned());
            }
            Ok(XmlEvent::End(element)) if element.name().as_ref() == tag.as_bytes() => {
                in_tag = false;
            }
            Ok(XmlEvent::Eof) => break,
            Ok(_) => {}
            Err(error) => return Err(error).context("failed to parse XML metadata"),
        }
    }

    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_neoforge_versions_to_minecraft_versions() {
        assert_eq!(
            minecraft_version_for_loader("21.0.114-beta"),
            Some("1.21".to_string())
        );
        assert_eq!(
            minecraft_version_for_loader("21.4.111-beta"),
            Some("1.21.4".to_string())
        );
        assert_eq!(
            minecraft_version_for_loader("26.1.2.31-beta"),
            Some("26.1.2".to_string())
        );
        assert_eq!(
            minecraft_version_for_loader("0.25w14craftmine.3-beta"),
            None
        );
    }

    #[test]
    fn matches_neoforge_versions_to_selected_minecraft_versions() {
        assert!(version_matches_minecraft("21.4.111-beta", "1.21.4"));
        assert!(version_matches_minecraft("26.1.2.31-beta", "26.1.2"));
        assert!(!version_matches_minecraft("21.4.111-beta", "1.21.5"));
    }

    #[test]
    fn parses_maven_metadata_versions() {
        let xml = r#"
            <metadata>
              <versioning>
                <versions>
                  <version>21.4.110-beta</version>
                  <version>21.4.111-beta</version>
                </versions>
              </versioning>
            </metadata>
        "#;

        assert_eq!(
            extract_xml_values(xml, "version").unwrap(),
            vec!["21.4.110-beta", "21.4.111-beta"]
        );
    }
}
