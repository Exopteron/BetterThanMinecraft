use crate::classic::*;
use crate::game::*;
pub struct EpicPlugin {}
use rlua::{Function, Lua, MetaMethod, Result, UserData, UserDataMethods, Variadic};
use rlua_async::ScopeExt;
use rlua_async::{ChunkExt, ContextExt};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::runtime::Handle;
#[derive(Clone)]
enum InLuaCommand {
    ChatBroadcast { text: String },
    ChatToID { id: i8, text: String },
    SetBlock { x: i16, y: i16, z: i16, id: u8 },
}
#[derive(Clone)]
struct ChatModule {}
#[derive(Clone)]
struct LuaBlock {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub id: u8,
}
impl UserData for LuaBlock {}
impl UserData for ChatModule {}
impl UserData for InLuaCommand {}
impl crate::game::Plugin for EpicPlugin {
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
        let mut lua_scripts = vec![];
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
        let mut lua_reg_commands = vec![];
        lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();
            let command_table = lua_ctx.create_table().unwrap();
            globals.set("command_table", command_table).unwrap();
            let loginfo = lua_ctx
                .create_function(|_, (string): (String)| {
                    log::info!("{}", string);
                    Ok(())
                })
                .unwrap();
            #[derive(Clone, Debug)]
            struct LuaCommand {
                command: String,
                args: String,
                desc: String,
                function: String,
            }
            impl UserData for LuaCommand {}
            let register_command = lua_ctx
                .create_function(
                    |lua_ctx, (command, args, desc, function): (String, String, String, String)| {
                        let globals = lua_ctx.globals();
                        let command_table: rlua::Table = globals.get("command_table").unwrap();
                        let len = command_table.len().unwrap();
                        command_table
                            .set(
                                len + 1,
                                LuaCommand {
                                    command,
                                    args,
                                    desc,
                                    function,
                                },
                            )
                            .unwrap();
                        Ok(())
                    },
                )
                .unwrap();
            globals.set("register_command", register_command).unwrap();
            // globals.set("loginfo", loginfo).unwrap();
            for script in lua_scripts {
                match lua_ctx.load(&script).exec().ok() {
                    Some(_) => {}
                    None => {
                        log::error!("An error occured executing a script.");
                    }
                }
            }
            let command_table: rlua::Table = globals.get("command_table").unwrap();
            for pair in command_table.pairs::<rlua::Value, LuaCommand>() {
                let (key, value) = pair.unwrap();
                lua_reg_commands.push(value);
                //log::info!("k, v: {:?} {:?}", key, value);
            }
        });
        for command in lua_reg_commands {
            let function = Arc::new(command.function);
            let handle = Handle::current();
            pre_gmts.register_command(
                command.command,
                &command.args,
                &command.desc,
                Box::new(move |gmts, args, sender| {
                    let function = function.clone();
                    let handle2 = handle.clone();
                    Box::pin(async move {
                        let handle2 = handle2.clone();
                        let future = tokio::task::spawn_blocking(move || {
                            let handle = handle2.clone();
                            let mut return_value = 0;
                            let lua = Lua::new();
                            let mut in_lua_cmds = vec![];
                            let handle = handle.clone();
                            lua.context(|lua_ctx| {
                                let globals = lua_ctx.globals();
                                let args_table = lua_ctx.create_table().unwrap();
                                let mut iternum = 1;
                                for argument in args {
                                    args_table.set(iternum, argument).unwrap();
                                    iternum += 1;
                                }
                                globals.set("cmd_args", args_table).unwrap();
                                let lua_cmds = lua_ctx.create_table().unwrap();
                                globals.set("lua_cmds", lua_cmds).unwrap();
                                globals.set("return_number", 0).unwrap();
                                globals.set("sender_id", sender).unwrap();
                                globals.set("chat", ChatModule {}).unwrap();
                                let chat_broadcast = lua_ctx
                                    .create_function(|lua_ctx, (text): (String)| {
                                        let globals = lua_ctx.globals();
                                        let lua_cmds: rlua::Table =
                                            globals.get("lua_cmds").unwrap();
                                        let len = lua_cmds.len().unwrap();
                                        lua_cmds
                                            .set(len + 1, InLuaCommand::ChatBroadcast { text })
                                            .unwrap();
                                        //in_lua_cmds.push(InLuaCommand::ChatBroadcast { text });
                                        Ok(())
                                    })
                                    .unwrap();
                                globals.set("chat_broadcast", chat_broadcast).unwrap();
                                let chat_to_id = lua_ctx
                                    .create_function(|lua_ctx, (id, text): (i8, String)| {
                                        let globals = lua_ctx.globals();
                                        let lua_cmds: rlua::Table =
                                            globals.get("lua_cmds").unwrap();
                                        let len = lua_cmds.len().unwrap();
                                        lua_cmds
                                            .set(len + 1, InLuaCommand::ChatToID { id, text })
                                            .unwrap();
                                        //in_lua_cmds.push(InLuaCommand::ChatBroadcast { text });
                                        Ok(())
                                    })
                                    .unwrap();
                                globals.set("chat_to_id", chat_to_id).unwrap();
                                let set_block = lua_ctx
                                    .create_function(
                                        |lua_ctx, (x, y, z, id): (i16, i16, i16, u8)| {
                                            let globals = lua_ctx.globals();
                                            let lua_cmds: rlua::Table =
                                                globals.get("lua_cmds").unwrap();
                                            let len = lua_cmds.len().unwrap();
                                            lua_cmds
                                                .set(
                                                    len + 1,
                                                    InLuaCommand::SetBlock { x, y, z, id },
                                                )
                                                .unwrap();
                                            //in_lua_cmds.push(InLuaCommand::ChatBroadcast { text });
                                            Ok(())
                                        },
                                    )
                                    .unwrap();
                                let gmts2 = gmts.clone();
                                let handle2 = handle.clone();
                                let get_username = lua_ctx
                                    .create_function(move |_ctx, (id): (i8)| {
                                        let handle = handle2.clone();
                                        let gmts = gmts2.clone();
                                        let gmts = gmts.clone();
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
                                    })
                                    .unwrap();
                                globals.set("get_username", get_username).unwrap();






                                let gmts2 = gmts.clone();
                                let handle2 = handle.clone();
                                let get_id = lua_ctx
                                    .create_function(move |_ctx, (username): (String)| {
                                        let handle = handle2.clone();
                                        let gmts = gmts2.clone();
                                        let gmts = gmts.clone();
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
                                    })
                                    .unwrap();
                                globals.set("get_id", get_id).unwrap();




                                let gmts2 = gmts.clone();
                                let handle2 = handle.clone();
                                let get_perm_level = lua_ctx
                                    .create_function(move |_ctx, (id): (i8)| {
                                        let handle = handle2.clone();
                                        let gmts = gmts2.clone();
                                        let gmts = gmts.clone();
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
                                    })
                                    .unwrap();
                                globals.set("get_perm_level", get_perm_level).unwrap();


                                let handle2 = handle.clone();
                                let gmts2 = gmts.clone();
                                let get_block = lua_ctx
                                    .create_function(move |_ctx, (x, y, z): (i16, i16, i16)| {
                                        let handle = handle2.clone();
                                        let gmts = gmts2.clone();
                                        let gmts = gmts.clone();
                                        let (tx, mut rx) = tokio::sync::oneshot::channel();
                                        handle.block_on(async move {
                                            tx.send(gmts.get_block(BlockPosition {x: x as usize, y: y as usize, z: z as usize}).await).unwrap();
                                        });

                                        let block: Block; //= gmts.get_username(id).await.unwrap();
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
                                    })
                                    .unwrap();
                                globals.set("get_block", get_block).unwrap();


                                globals.set("set_block", set_block).unwrap();
                                //let gmts = gmts.clone();
                                /*                             lua_ctx.scope(|scope| {
                                                                let gmts = gmts.clone();
                                                                lua_ctx.globals().set(
                                                                    "get_username",
                                                                    scope.create_async_function(lua_ctx, move |_ctx, (id): (i8)| {
                                                                        let gmts = gmts.clone();
                                                                        async move {
                                                                        let gmts = gmts.clone();
                                                                        log::info!("aeiou");
                                /*                                         let (tx, mut rx) = tokio::sync::oneshot::channel();
                                                                        handle.spawn(async move {
                                                                            log::info!("n");
                                                                            tx.send(gmts.get_username(id).await).unwrap();
                                                                            log::info!("a");
                                                                        }); */
                                                                        let username: String = gmts.get_username(id).await.unwrap();
                                /*                                         loop {
                                                                            let x = match rx.try_recv() {
                                                                                Ok(v) => v,
                                                                                Err(_) => {
                                                                                    continue;
                                                                                }
                                                                            };
                                                                            username = x.unwrap();
                                                                            break;
                                                                        } */
                                                                        return Ok(username);
                                                                    }}).unwrap(),
                                                                ).unwrap();
                                                                lua_ctx.load(function.as_bytes()).exec().unwrap();
                                                            }); */
                                lua_ctx.load(function.as_bytes()).exec().unwrap();
                                let lua_cmds: rlua::Table = globals.get("lua_cmds").unwrap();
                                for pair in lua_cmds.pairs::<rlua::Value, InLuaCommand>() {
                                    let (key, value) = pair.unwrap();
                                    in_lua_cmds.push(value);
                                    //log::info!("k, v: {:?} {:?}", key, value);
                                }
                                return_value = globals.get::<_, usize>("return_number").unwrap();
                            });
                            for command in in_lua_cmds {
                                match command {
                                    InLuaCommand::ChatBroadcast { text } => {
                                        let gmts = gmts.clone();
                                        handle.block_on(async move {
                                            gmts.chat_broadcast(&format!("{}", text), -1).await;
                                        });
                                    }
                                    InLuaCommand::ChatToID { id, text } => {
                                        let gmts = gmts.clone();
                                        handle.block_on(async move {
                                            gmts.chat_to_id(&format!("{}", text), -1, id).await;
                                        });
                                    }
                                    InLuaCommand::SetBlock { x, y, z, id } => {
                                        let gmts = gmts.clone();
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
                                    }
                                }
                            }
                            return_value
                        });
                        future.await.unwrap()
                    })
                }),
            );
            //log::info!("Command: {:?}", command);
        }
        /*         pre_gmts.cpe_handler.custom_block_support_level(1);
        pre_gmts.register_command(
            "holdblock".to_string(),
            "",
            "Change your held block",
            Box::new(move |gmts, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 4 {
                            let held_block = if let Some(b) = gmts.get_held_block(sender as i8).await {
                                b
                            } else {
                                return 3;
                            };
                            gmts.chat_to_id(&format!("Your current held block is {}.", held_block), -1, sender).await;
                            gmts.update_held_block(sender as i8, 1, false).await;
                        } else {
                            return 2;
                        }
                    } else {
                        return 3;
                    };
                    0
                })
            }),
        ); */
    }
}
