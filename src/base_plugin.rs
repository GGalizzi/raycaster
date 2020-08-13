use bevy::prelude::{AppBuilder, Plugin};
pub struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(bevy::type_registry::TypeRegistryPlugin::default());
        app.add_plugin(bevy::core::CorePlugin::default());
        app.add_plugin(bevy::transform::TransformPlugin::default());
        app.add_plugin(bevy::diagnostic::DiagnosticsPlugin::default());
        app.add_plugin(bevy::asset::AssetPlugin::default());
        app.add_plugin(bevy::scene::ScenePlugin::default());
    }
}
