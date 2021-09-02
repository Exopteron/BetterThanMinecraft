use crate::classic::*;
use crate::game::*;
pub struct LuaPluginAPI {}
use rlua::{Function, Lua, MetaMethod, Result, UserData, UserDataMethods, Variadic};
use rlua_async::ScopeExt;
use rlua_async::{ChunkExt, ContextExt};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::runtime::Handle;
use once_cell::sync::Lazy;
struct SendableLua(Lua);
unsafe impl Send for SendableLua {

}
impl crate::game::Plugin for LuaPluginAPI {
    fn initialize(pre_gmts: &mut PreGMTS) {
        let mut path_vec: Vec<std::path::PathBuf> = Vec::new();
        for element in std::path::Path::new(r"plugins/").read_dir().unwrap() {
            let path = element.unwrap().path();
            if let Some(extension) = path.extension() {
                if extension == "lua" {
                    path_vec.push(path);
                }
            }
        }
        let mut lua_scripts = Vec::with_capacity(path_vec.len());
        for path in path_vec {
            log::info!("Loading plugin from {}.", path.to_string_lossy());
            let file = if let Some(f) = std::fs::read_to_string(path).ok() {
                f
            } else {
                continue;
            };
            lua_scripts.push(file);
        }
        let pl_count = lua_scripts.len();
        match pl_count {
            1 => {
                log::info!("Successfully loaded {} plugin.", pl_count);
            }
            _ => {
                log::info!("Successfully loaded {} plugins.", pl_count);
            }
        }
        let lua = Lua::new();
        lua.context(|lua_ctx| {
            lua_ctx.scope(|scope| {
                let gmts_module = lua_ctx.create_table().unwrap();
                gmts_module.set(
                    "register_command",
                    scope.create_function_mut(|_, (command, args, desc, function): (String, String, String, String)| {
/*                         let lua = Lua::new();
                        lua.context(|lua_ctx| {
                            let globals = lua_ctx.globals();
                            globals.set("command_function", function);
                        }); */
                        //let lua = lua.clone();
                        //let lua = Arc::new(std::sync::Mutex::new(lua));
                        let function = Arc::new(function);
                        pre_gmts.register_command(
                            command,
                            &args,
                            &desc,
                            Box::new(move |gmts, args, sender| {
                                let function = function.clone();
                                let gmts = gmts.clone();
                                Box::pin(async move {
                                    let gmts = gmts.clone();
                                    let function = function.clone();
                                    let handle = Handle::current();
                                    let future = tokio::task::spawn_blocking(move || {
                                        let gmts = gmts.clone();
                                        let function = function.clone();
                                        let lua = Lua::new();
                                        let mut return_value = 0;
                                        //let lua = Arc::try_unwrap(lua).ok().unwrap().into_inner().unwrap();
                                        lua.context(|lua_ctx| {
                                            let gmts = gmts.clone();
                                            let globals = lua_ctx.globals();
                                            let args_table = lua_ctx.create_table().unwrap();
                                            let mut iternum = 1;
                                            for argument in args {
                                                args_table.set(iternum, argument).unwrap();
                                                iternum += 1;
                                            }
                                            globals.set("cmd_args", args_table).unwrap();
                                            let executor = lua_ctx.scope(|scope| {
                                                globals.set("return_number", 0).unwrap();
                                                globals.set("sender_id", sender).unwrap();
                                                // Logger module
                                                let logger_module = lua_ctx.create_table().unwrap();
                                                logger_module.set("info", scope.create_function_mut(|_, message: String| {
                                                    log::info!("{}", message);
                                                    Ok(())
                                                }).unwrap()).unwrap();
                                                logger_module.set("warn", scope.create_function_mut(|_, message: String| {
                                                    log::warn!("{}", message);
                                                    Ok(())
                                                }).unwrap()).unwrap();
                                                logger_module.set("error", scope.create_function_mut(|_, message: String| {
                                                    log::error!("{}", message);
                                                    Ok(())
                                                }).unwrap()).unwrap();
                                                logger_module.set("debug", scope.create_function_mut(|_, message: String| {
                                                    log::debug!("{}", message);
                                                    Ok(())
                                                }).unwrap()).unwrap();
                                                globals.set("logger", logger_module).unwrap();

                                                // Chat module
                                                let gmts2 = gmts.clone();
                                                let handle2 = handle.clone();
                                                let chat_module = lua_ctx.create_table().unwrap();
                                                chat_module.set("broadcast", scope.create_function_mut(move |_, message: String| {
                                                    let gmts = gmts2.clone();
                                                    let handle = handle2.clone();
                                                    handle.block_on(async move {
                                                        gmts.chat_broadcast(&message, -1).await
                                                    });
                                                    Ok(())
                                                }).unwrap()).unwrap();
                                                let gmts2 = gmts.clone();
                                                let handle2 = handle.clone();
                                                chat_module.set("send_to_id", scope.create_function_mut(move |_, (id, message): (i8, String)| {
                                                    let gmts = gmts2.clone();
                                                    let handle = handle2.clone();
                                                    handle.block_on(async move {
                                                        gmts.chat_to_id(&message, -1, id).await
                                                    });
                                                    Ok(())
                                                }).unwrap()).unwrap();
                                                globals.set("chat", chat_module).unwrap();

                                                // World module
                                                let gmts2 = gmts.clone();
                                                let handle2 = handle.clone();
                                                let world_module = lua_ctx.create_table().unwrap();
                                                world_module.set("set_block", scope.create_function_mut(move |_, (x, y, z, id): (i16, i16, i16, u8)| {
                                                    let gmts = gmts2.clone();
                                                    let handle = handle2.clone();
                                                    handle.block_on(async move {
                                                        gmts.set_block(
                                                            Block {
                                                                position: BlockPosition {
                                                                    x: x as usize,
                                                                    y: y as usize,
                                                                    z: z as usize,
                                                                },
                                                                id,
                                                            },
                                                            -69,
                                                        )
                                                        .await;
                                                    });
                                                    Ok(())
                                                }).unwrap()).unwrap();
                                                let gmts2 = gmts.clone();
                                                let handle2 = handle.clone();
                                                world_module.set("get_block", scope.create_function_mut(move |_, (x, y, z): (i16, i16, i16)| {
                                                    let gmts = gmts2.clone();
                                                    let handle = handle2.clone();
                                                    let (tx, mut rx) = tokio::sync::oneshot::channel();
                                                    handle.block_on(async move {
                                                        tx.send(gmts.get_block(BlockPosition {x: x as usize, y: y as usize, z: z as usize}).await).unwrap();
                                                    });
            
                                                    let block: Block;
                                                    loop {
                                                        let x = match rx.try_recv() {
                                                            Ok(v) => v,
                                                            Err(_) => {
                                                                continue;
                                                            }
                                                        };
                                                        block = x.unwrap();
                                                        break;
                                                    }
                                                    return Ok(block.id);
                                                }).unwrap()).unwrap();
                                                globals.set("world", world_module).unwrap();
                                                // Players module
                                                let gmts2 = gmts.clone();
                                                let handle2 = handle.clone();
                                                let players_module = lua_ctx.create_table().unwrap();
                                                players_module.set("get_username", scope.create_function_mut(move |_, id: i8| {
                                                    let gmts = gmts2.clone();
                                                    let handle = handle2.clone();
                                                    let (tx, mut rx) = tokio::sync::oneshot::channel();
                                                    handle.block_on(async move {
                                                        tx.send(gmts.get_username(id).await).unwrap();
                                                    });
            
                                                    let username: String; //= gmts.get_username(id).await.unwrap();
                                                    loop {
                                                        let x = match rx.try_recv() {
                                                            Ok(v) => v,
                                                            Err(_) => {
                                                                continue;
                                                            }
                                                        };
                                                        username = x.unwrap();
                                                        break;
                                                    }
                                                    return Ok(username);
                                                }).unwrap()).unwrap();
                                                let gmts2 = gmts.clone();
                                                let handle2 = handle.clone();
                                                players_module.set("get_id", scope.create_function_mut(move |_, username: String| {
                                                    let gmts = gmts2.clone();
                                                    let handle = handle2.clone();
                                                    let (tx, mut rx) = tokio::sync::oneshot::channel();
                                                    handle.block_on(async move {
                                                        tx.send(gmts.get_id(username).await).unwrap();
                                                    });
            
                                                    let id: i8; //= gmts.get_username(id).await.unwrap();
                                                    loop {
                                                        let x = match rx.try_recv() {
                                                            Ok(v) => v,
                                                            Err(_) => {
                                                                continue;
                                                            }
                                                        };
                                                        id = x.unwrap();
                                                        break;
                                                    }
                                                    return Ok(id);
                                                }).unwrap()).unwrap();
                                                let gmts2 = gmts.clone();
                                                let handle2 = handle.clone();
                                                players_module.set("perm_level", scope.create_function_mut(move |_, id: i8| {
                                                    let gmts = gmts2.clone();
                                                    let handle = handle2.clone();
                                                    let (tx, mut rx) = tokio::sync::oneshot::channel();
                                                    handle.block_on(async move {
                                                        tx.send(gmts.get_permission_level(id).await).unwrap();
                                                    });
            
                                                    let p_level: usize; //= gmts.get_username(id).await.unwrap();
                                                    loop {
                                                        let x = match rx.try_recv() {
                                                            Ok(v) => v,
                                                            Err(_) => {
                                                                continue;
                                                            }
                                                        };
                                                        p_level = x.unwrap();
                                                        break;
                                                    }
                                                    return Ok(p_level);
                                                }).unwrap()).unwrap();
                                                globals.set("players", players_module).unwrap();

                                                // Storage module
                                                let gmts2 = gmts.clone();
                                                let handle2 = handle.clone();
                                                let storage_module = lua_ctx.create_table().unwrap();
                                                storage_module.set("new_value", scope.create_function_mut(move |_, (key, value): (String, String) | {
                                                    let gmts = gmts2.clone();
                                                    let handle = handle2.clone();
                                                    handle.block_on(async move {
                                                        gmts.new_value(&key, GMTSElement { val: Arc::new(Box::new(value)) }).await;
                                                    });
                                                    Ok(())
                                                }).unwrap()).unwrap();
                                                let gmts2 = gmts.clone();
                                                let handle2 = handle.clone();
                                                storage_module.set("get_value", scope.create_function_mut(move |_, key: String| {
                                                    let gmts = gmts2.clone();
                                                    let handle = handle2.clone();
                                                    let (tx, mut rx) = tokio::sync::oneshot::channel();
                                                    handle.block_on(async move {
                                                        tx.send(gmts.get_value(&key).await).ok().unwrap();
                                                    });
            
                                                    let p_level: GMTSElement; //= gmts.get_username(id).await.unwrap();
                                                    loop {
                                                        let x = match rx.try_recv() {
                                                            Ok(v) => v,
                                                            Err(_) => {
                                                                continue;
                                                            }
                                                        };
                                                        let x = match x {
                                                            Some(x) => x,
                                                            None => {
                                                                return Err(rlua::Error::external(std::io::Error::new(std::io::ErrorKind::Other, "Value does not exist!")));
                                                            }
                                                        };
                                                        p_level = x;
                                                        break;
                                                    }
                                                let p_level = if let Some(l) =
                                                    p_level.val.downcast_ref::<String>()
                                                {
                                                    l
                                                } else {
                                                    return Err(rlua::Error::external(std::io::Error::new(std::io::ErrorKind::Other, "Not accessible from Lua!")));
                                                };
                                                    return Ok(p_level.clone());
                                                }).unwrap()).unwrap();
                                                globals.set("storage", storage_module).unwrap();


                                                lua_ctx.load(function.as_bytes()).exec()
                                            });
                                            return_value = globals.get::<_, usize>("return_number").unwrap();
                                            match executor {
                                                Ok(_) => {},
                                                Err(e) => {
                                                    log::error!("Script error! Details: {:?}", e);
                                                    return_value = 3;
                                                }
                                            }
                                        });
                                        return_value
                                    });
                                    future.await.unwrap()
                                })
                            }),
                        );
                        Ok(())
                    }).unwrap()
                ).unwrap();
                lua_ctx.globals().set("game", gmts_module).unwrap();
                lua_ctx.load(lua_scripts[0].as_bytes()).exec().unwrap();
            });
/*             lua_ctx.create_function(|_, (command, args, desc, function): (String, String, String, Function)| {
                
            }); */
        });

    }
}