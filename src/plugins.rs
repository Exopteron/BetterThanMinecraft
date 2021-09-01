pub mod testplugin;
pub mod longermessages;
pub mod coreutils;
pub mod anticheat;
pub mod cpe;
pub mod epicplugin;
use crate::game::*;
use crate::classic::*;
pub struct PluginManager {}
use std::sync::Arc;
impl crate::game::Plugin for PluginManager {
    fn initialize(pre_gmts: &mut PreGMTS) {
        epicplugin::EpicPlugin::initialize(pre_gmts);
    }
}