use crate::{Curio, Formation, Identity, Severity, SystemComponent};

pub struct CurioBuilder {
    pub(crate) metadata: Identity,
    pub(crate) plugins: Vec<Box<dyn SystemComponent>>,
    pub(crate) plugin_paths: Vec<String>,
    pub(crate) gamemode: Formation,
}
impl CurioBuilder {
    pub fn set_game_mode(mut self, gamemode: Formation) -> Self {
        self.gamemode = gamemode;
        self
    }
    pub fn add_plugin(mut self, plugin: Box<dyn SystemComponent>) -> Self {
        self.plugins.push(plugin);
        self
    }
    pub fn add_plugin_path(mut self, path: &str) -> Self {
        self.plugin_paths.push(path.to_string());
        self
    }
    pub fn set_metadata(mut self, metadata: Identity) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn imbue(self) -> Curio {
        // log for visibility
        Curio::log(Severity::Info, &format!("Imbuing: {} v{}", self.metadata.name, self.metadata.version));
        // create the curio
        Curio::new(self)
    }
}
