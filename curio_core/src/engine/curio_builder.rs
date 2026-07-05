use crate::{Curio, Formation, Identity, PluginCommon, Severity};

/// Builder for setting up a Curio. Complete by running Imbue
pub struct CurioBuilder {
    pub(crate) metadata: Identity,
    pub(crate) plugins: Vec<Box<dyn PluginCommon>>,
    pub(crate) gamemode: Formation,
}
impl CurioBuilder {
    // Set the identity of the Curio
    pub fn identity(mut self, metadata: Identity) -> Self {
        self.metadata = metadata;
        self
    }
    /// Set the formation of the Curio
    pub fn formation(mut self, gamemode: Formation) -> Self {
        self.gamemode = gamemode;
        self
    }
    /// Add a plugin to be used
    pub fn plugin(mut self, plugin: Box<dyn PluginCommon>) -> Self {
        self.plugins.push(plugin);
        self
    }
    /// Finalize and create a Curio from set parameters
    pub fn imbue(self) -> Curio {
        // log for visibility
        Curio::log(Severity::Info, &format!("Imbuing: {} v{}", self.metadata.name, self.metadata.version));
        // create the curio
        Curio::new(self)
    }
}
