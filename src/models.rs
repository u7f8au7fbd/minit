#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Loader {
    Fabric,
    NeoForge,
}

impl Loader {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fabric => "Fabric",
            Self::NeoForge => "NeoForge",
        }
    }
}

#[derive(Debug)]
pub struct InstallChoice {
    pub loader: Loader,
    pub minecraft_version: String,
    pub loader_version: String,
    pub installer_url: String,
}
