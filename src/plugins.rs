pub mod testplugin;
pub mod longermessages;
pub mod coreutils;
pub mod anticheat;
pub mod cpe;
pub mod epicplugin;
pub mod lua_plugin_api;
use crate::game::*;
use crate::classic::*;
pub struct PluginManager {}
use std::sync::Arc;
impl crate::game::Plugin for PluginManager {
    fn initialize(pre_gmts: &mut PreGMTS) {
        //epicplugin::EpicPlugin::initialize(pre_gmts);
        lua_plugin_api::LuaPluginAPI::initialize(pre_gmts);
    }
}