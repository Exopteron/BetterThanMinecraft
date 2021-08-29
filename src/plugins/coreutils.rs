use crate::classic::*;
use crate::game::*;
use crate::settings;
pub struct CoreUtils {}
use crate::CONFIGURATION;
use std::sync::Arc;
/*

Command plan:
/ban - ban user
/unban - unban user
/tp - tp user to user
/tppos - tp user to position
/say - say message
*/
impl crate::game::Plugin for CoreUtils {
    fn initialize(pre_gmts: &mut PreGMTS) {
        pre_gmts.register_oninitialize(Box::new(move |gmts: CMDGMTS| {
            Box::pin(async move {
                let pos = gmts.get_spawnpos().await?;
                gmts.new_value(
                    "Coreutils_SpawnPosition",
                    GMTSElement {
                        val: Arc::new(Box::new(pos.clone())),
                    },
                )
                .await;
                gmts.new_value(
                    "Coreutils_Whitelist",
                    GMTSElement {
                        val: Arc::new(Box::new((CONFIGURATION.whitelist_enabled, settings::get_whitelist()))),
                    },
                )
                .await;
                Some(())
            })
        }));
        pre_gmts.register_early_onconnect_hook(Box::new(|gmts, stream, id| {
            Box::pin(async move {
                use tokio::io::AsyncWriteExt;
                let username = gmts.get_username(id).await?;
                let banlist = settings::get_banlist();
                let mut stream = stream.lock().await;
                for ban in banlist {
                    if ban.username == username {
                        let packet = crate::classic::Packet::Disconnect {
                            reason: ban.reason.clone(),
                        };
                        stream
                            .write_all(&ClassicPacketWriter::serialize(packet).ok()?)
                            .await
                            .ok()?;
                        log::info!("{} is banned for {}!", ban.username, ban.reason);
                        return None;
                    }
                }
                return Some(());
            })
        }));
        pre_gmts.register_command(
            "list".to_string(),
            "",
            "Get player list",
            Box::new(move |gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 1 {
                            let online_players = if let Some(c) = gmts.player_count().await {
                                c
                            } else {
                                return 3;
                            };
                            gmts.chat_to_id(&format!("&7There are &c{}&7 out of a maximum of &c{}&7 players online.", online_players, CONFIGURATION.max_players), -1, sender).await;
                        }
                    } else {
                        return 3;
                    };
                    0
                })
            }),
        );
        pre_gmts.register_command(
            "help".to_string(),
            "",
            "Get command help",
            Box::new(move |gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 1 {
                            let all_cmds = gmts.get_commands_list().await;
                            gmts.chat_to_id("&fHelp:", -1, sender).await;
                            let mut to_sort_alphabetical = std::collections::BTreeMap::new();
                            for (name, data) in all_cmds {
                                to_sort_alphabetical.insert(name.clone(), format!("&c/{} {} &f- &7{}", name, data.args, data.desc));
                            }
                            for (_, message) in to_sort_alphabetical {
                                let message =
                                    message
                                        .as_bytes()
                                        .to_vec();
                                let message = message.chunks(60).collect::<Vec<&[u8]>>();
                                for message in message {
                                    gmts.chat_to_id(
                                        &format!(
                                            "&7{}",
                                            String::from_utf8_lossy(message).to_string()
                                        ),
                                        -1,
                                        sender,
                                    )
                                    .await;
                                }
                            }
                        }
                    } else {
                        return 3;
                    };
                    0
                })
            }),
        );
        pre_gmts.register_command(
            "setworldspawn".to_string(),
            "",
            "Set the world spawnpoint to your current position.",
            Box::new(move |gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 4 {
                            let position = if let Some(p) = gmts.get_position(sender).await {
                                p
                            } else {
                                return 3;
                            };
                            gmts.set_value(
                                "Coreutils_SpawnPosition",
                                GMTSElement {
                                    val: Arc::new(Box::new(position.clone())),
                                },
                            )
                            .await;
                            gmts.set_spawnpos(position).await;
                            gmts.msg_broadcast(PlayerCommand::SpawnPlayer {
                                position: position.clone(),
                                id: -1,
                                name: "".to_string(),
                            })
                            .await;
                            let our_name = if let Some(n) = gmts.get_username(sender).await {
                                n
                            } else {
                                return 3;
                            };
                            gmts.chat_to_permlevel(
                                &format!(
                                    "&d[{}: Set the spawnpoint to {} {} {}]",
                                    our_name,
                                    position.x / 32,
                                    position.y / 32,
                                    position.z / 32
                                ),
                                -1,
                                4,
                            )
                            .await;
                        } else {
                            return 2;
                        }
                    } else {
                        return 3;
                    };
                    0
                })
            }),
        );
        pre_gmts.register_command(
            "tp".to_string(),
            "(persona) (personb)",
            "teleport person a to person b.",
            Box::new(move |gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 4 {
                            let our_name = if let Some(n) = gmts.get_username(sender).await {
                                n
                            } else {
                                return 1;
                            };
                            if args.len() < 2 {
                                return 1;
                            }
                            let id_a = if let Some(i) = gmts.get_id(args[0].to_string()).await {
                                i
                            } else {
                                return 1;
                            };
                            let id_b = if let Some(i) = gmts.get_id(args[1].to_string()).await {
                                i
                            } else {
                                return 1;
                            };
                            let pos = if let Some(p) = gmts.get_position(id_b).await {
                                p
                            } else {
                                return 3;
                            };
                            if let None = gmts.tp_id_pos(id_a, pos).await {
                                return 3;
                            }
                            gmts.chat_to_permlevel(
                                &format!(
                                    "&d[{}: Teleporting {} to {}]",
                                    our_name, args[0], args[1]
                                ),
                                -1,
                                4,
                            )
                            .await;
                        }
                    } else {
                        return 3;
                    };
                    0
                })
            }),
        );
        pre_gmts.register_command(
            "permlevel".to_string(),
            "",
            "Get your permission level",
            Box::new(move |gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        gmts.chat_to_id(&format!("Your permission level is {}.", p), -1, sender)
                            .await;
                    } else {
                        return 3;
                    };
                    0
                })
            }),
        );
        pre_gmts.register_command(
            "say".to_string(),
            "(text)",
            "Broadcast a message.",
            Box::new(move |gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 4 {
                            let username = match gmts.get_username(sender).await {
                                Some(u) => u,
                                None => {
                                    return 3;
                                }
                            };
                            gmts.chat_broadcast(
                                &format!("&d[{}] {}", username, args.join(" ")),
                                -1,
                            )
                            .await;
                        } else {
                            return 2;
                        }
                    } else {
                        return 3;
                    };
                    0
                })
            }),
        );
        pre_gmts.register_command(
            "me".to_string(),
            "(text)",
            "add desc here",
            Box::new(move |gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 1 {
                            let username = match gmts.get_username(sender).await {
                                Some(u) => u,
                                None => {
                                    return 3;
                                }
                            };
                            gmts.chat_broadcast(
                                &format!("&5* {} {}", username, args.join(" ")),
                                -1,
                            )
                            .await;
                        } else {
                            return 2;
                        }
                    } else {
                        return 3;
                    };
                    0
                })
            }),
        );
        pre_gmts.register_command(
            "msg".to_string(),
            "(player) (message)",
            "Send a private message to another player",
            Box::new(|gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 1 {
                            if args.len() < 1 {
                                return 1;
                            }
                            let our_name = if let Some(n) = gmts.get_username(sender).await {
                                n
                            } else {
                                return 1;
                            };
                            if let None = gmts.get_id(args[0].clone()).await {
                                return 1;
                            }
                            let msg = &format!(
                                "&8[&cme &8-> &c{}&8]&7 {}",
                                args[0].clone(),
                                args[1..].join(" ")
                            );
                            gmts.chat_to_id(msg, -1, sender).await;
                            let msg = &format!(
                                "&8[&c{} &8-> &cme&8]&7 {}",
                                our_name,
                                args[1..].join(" ")
                            );
                            // do really cool things
                            gmts.chat_to_username(&msg, -1, args[0].clone()).await;
                        }
                    }
                    0
                })
            }),
        );
        pre_gmts.register_command(
            "stop".to_string(),
            "",
            "Save the world file and stop the server.",
            Box::new(move |gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 4 {
                            let our_name = if let Some(n) = gmts.get_username(sender).await {
                                n
                            } else {
                                return 3;
                            };
                            gmts.chat_to_permlevel(
                                &format!("&d[{}: Stopping the server...]", our_name),
                                -1,
                                4,
                            )
                            .await;
                            if let None = gmts.stop_server().await {
                                gmts.chat_to_permlevel(
                                    &format!("&d[{}: Error stopping the server.]", our_name),
                                    -1,
                                    4,
                                )
                                .await;
                                return 3;
                            }
                        } else {
                            return 2;
                        }
                    } else {
                        return 3;
                    };
                    0
                })
            }),
        );
        pre_gmts.register_command(
            "save-all".to_string(),
            "",
            "Save the world file.",
            Box::new(move |gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 4 {
                            let our_name = if let Some(n) = gmts.get_username(sender).await {
                                n
                            } else {
                                return 3;
                            };
                            gmts.chat_to_permlevel(
                                &format!("&d[{}: Forcing save..]", our_name),
                                -1,
                                4,
                            )
                            .await;
                            if let None = gmts.save_world().await {
                                gmts.chat_to_permlevel(
                                    &format!("&d[{}: Error saving the world.]", our_name),
                                    -1,
                                    4,
                                )
                                .await;
                                return 3;
                            }
                            gmts.chat_to_permlevel(
                                &format!("&d[{}: Save complete.]", our_name),
                                -1,
                                4,
                            )
                            .await;
                        } else {
                            return 2;
                        }
                    } else {
                        return 3;
                    };
                    0
                })
            }),
        );
        pre_gmts.register_command(
            "whitelist".to_string(),
            "(on,off,add,remove,list)",
            "Control the whitelist.",
            Box::new(move |gmts: CMDGMTS, mut args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 4 {
                            if args.len() < 1 {
                                return 1;
                            }
                            let arg = args[0].clone();
                            args.remove(0);
                            match arg.as_str() {
                                "add" => {
                                    if args.len() < 1 {
                                        return 1;
                                    }
                                    let our_name = if let Some(n) = gmts.get_username(sender).await {
                                        n
                                    } else {
                                        return 3;
                                    };
                                    let x = if let Some(l) = gmts.get_value("Coreutils_Whitelist").await {
                                        l
                                    } else {
                                        return 3;
                                    };
                                    let whitelist = if let Some(l) = x.val.downcast_ref::<(bool, Vec<String>)>() {
                                        l
                                    } else {
                                        return 3;
                                    };
                                    let (whitelist_enabled, mut whitelist) = whitelist.clone();
                                    for name in &whitelist {
                                        if &args[0] == name {
                                            gmts.chat_to_id(
                                                "User is already whitelisted!",
                                                -1,
                                                sender,
                                            )
                                            .await;
                                            return 0;
                                        }
                                    }
                                    whitelist.push(args[0].clone());
                                    gmts.set_value(
                                        "Coreutils_Whitelist",
                                        GMTSElement {
                                            val: Arc::new(Box::new((whitelist_enabled, whitelist))),
                                        },
                                    )
                                    .await;
                                    gmts.chat_to_permlevel(
                                        &format!("&d[{}: Adding {} to the whitelist.]", our_name, args[0]),
                                        -1,
                                        4,
                                    )
                                    .await;
                                    settings::add_whitelist(&args[0]);
                                }
                                "remove" => {
                                    if args.len() < 1 {
                                        return 1;
                                    }
                                    let our_name = if let Some(n) = gmts.get_username(sender).await {
                                        n
                                    } else {
                                        return 3;
                                    };
                                    let x = if let Some(l) = gmts.get_value("Coreutils_Whitelist").await {
                                        l
                                    } else {
                                        return 3;
                                    };
                                    let whitelist = if let Some(l) = x.val.downcast_ref::<(bool, Vec<String>)>() {
                                        l
                                    } else {
                                        return 3;
                                    };
                                    let (whitelist_enabled, mut whitelist) = whitelist.clone();
                                    whitelist.retain(|name| {
                                        &args[0] != name
                                    });
                                    gmts.set_value(
                                        "Coreutils_Whitelist",
                                        GMTSElement {
                                            val: Arc::new(Box::new((whitelist_enabled, whitelist))),
                                        },
                                    )
                                    .await;
                                    gmts.chat_to_permlevel(
                                        &format!("&d[{}: Removing {} from the whitelist.]", our_name, args[0]),
                                        -1,
                                        4,
                                    )
                                    .await;
                                    settings::remove_whitelist(&args[0]);
                                }
                                "list" => {
                                    let x = if let Some(l) = gmts.get_value("Coreutils_Whitelist").await {
                                        l
                                    } else {
                                        return 3;
                                    };
                                    let whitelist = if let Some(l) = x.val.downcast_ref::<(bool, Vec<String>)>() {
                                        l
                                    } else {
                                        return 3;
                                    };
                                    let (whitelist_enabled, whitelist) = whitelist.clone();
                                    gmts.chat_to_id(
                                        &format!("Whitelist enabled: &7{}", whitelist_enabled),
                                        -1,
                                        sender,
                                    )
                                    .await;
                                    gmts.chat_to_id(
                                        "Whitelisted users:",
                                        -1,
                                        sender,
                                    )
                                    .await;
                                    for name in whitelist {
                                        gmts.chat_to_id(
                                            &format!("&7-- {}", name),
                                            -1,
                                            sender,
                                        )
                                        .await;
                                    }
                                }
                                "on" => {
                                    let our_name = if let Some(n) = gmts.get_username(sender).await {
                                        n
                                    } else {
                                        return 3;
                                    };
                                    let x = if let Some(l) = gmts.get_value("Coreutils_Whitelist").await {
                                        l
                                    } else {
                                        return 3;
                                    };
                                    let whitelist = if let Some(l) = x.val.downcast_ref::<(bool, Vec<String>)>() {
                                        l
                                    } else {
                                        return 3;
                                    };
                                    let (whitelist_enabled, whitelist) = whitelist.clone();
                                    if !whitelist_enabled {
                                        gmts.set_value(
                                            "Coreutils_Whitelist",
                                            GMTSElement {
                                                val: Arc::new(Box::new((true, whitelist))),
                                            },
                                        )
                                        .await;
                                        gmts.chat_to_permlevel(
                                            &format!("&d[{}: Turning on the whitelist]", our_name),
                                            -1,
                                            4,
                                        )
                                        .await;
                                    } else {
                                        gmts.chat_to_id(
                                            "Whitelist is already enabled!",
                                            -1,
                                            sender,
                                        )
                                        .await;
                                        return 0;
                                    }
                                }
                                "off" => {
                                    let our_name = if let Some(n) = gmts.get_username(sender).await {
                                        n
                                    } else {
                                        return 3;
                                    };
                                    let x = if let Some(l) = gmts.get_value("Coreutils_Whitelist").await {
                                        l
                                    } else {
                                        return 3;
                                    };
                                    let whitelist = if let Some(l) = x.val.downcast_ref::<(bool, Vec<String>)>() {
                                        l
                                    } else {
                                        return 3;
                                    };
                                    let (whitelist_enabled, whitelist) = whitelist.clone();
                                    if whitelist_enabled {
                                        gmts.set_value(
                                            "Coreutils_Whitelist",
                                            GMTSElement {
                                                val: Arc::new(Box::new((false, whitelist))),
                                            },
                                        )
                                        .await;
                                        gmts.chat_to_permlevel(
                                            &format!("&d[{}: Turning off the whitelist]", our_name),
                                            -1,
                                            4,
                                        )
                                        .await;
                                    } else {
                                        gmts.chat_to_id(
                                            "Whitelist is already disabled!",
                                            -1,
                                            sender,
                                        )
                                        .await;
                                        return 0;
                                    }
                                }
                                _ => {
                                    return 1;
                                }
                            }
                        } else {
                            return 2;
                        }
                    } else {
                        return 3;
                    };
                    0
                })
            }),
        );
        pre_gmts.register_command(
            "unban".to_string(),
            "(player)",
            "Unban a user from the game.",
            Box::new(move |gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 4 {
                            if args.len() < 1 {
                                return 1;
                            }
                            let our_name = if let Some(n) = gmts.get_username(sender).await {
                                n
                            } else {
                                return 3;
                            };
                            gmts.chat_to_permlevel(
                                &format!("&d[{}: Unbanning user {}]", our_name, args[0]),
                                -1,
                                4,
                            )
                            .await;
                            settings::remove_banlist(&args[0]);
                        } else {
                            return 2;
                        }
                    } else {
                        return 3;
                    };
                    0
                })
            }),
        );
        pre_gmts.register_command(
            "ban".to_string(),
            "(player) (reason)",
            "Ban a user from the game.",
            Box::new(move |gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 4 {
                            if args.len() < 1 {
                                return 1;
                            }
                            let their_id = if let Some(i) = gmts.get_id(args[0].to_string()).await {
                                i
                            } else {
                                let reason = &args[1..].join(" ");
                                let our_name = if let Some(n) = gmts.get_username(sender).await {
                                    n
                                } else {
                                    return 3;
                                };
                                gmts.chat_to_permlevel(
                                    &format!("&d[{}: Banning user {}]", our_name, args[0]),
                                    -1,
                                    4,
                                )
                                .await;
                                gmts.kick_user_by_name(&args[0], reason).await;
                                settings::add_banlist(&args[0], &reason.clone());
                                return 0;
                            };
                            let their_p_level = match gmts.get_permission_level(their_id).await {
                                Some(x) => x,
                                None => {
                                    return 1;
                                }
                            };
                            if their_p_level > p {
                                gmts.chat_to_id(
                                    "Can't kick a user with higher permissions.",
                                    -1,
                                    sender,
                                )
                                .await;
                            } else {
                                let reason = &args[1..].join(" ");
                                let our_name = if let Some(n) = gmts.get_username(sender).await {
                                    n
                                } else {
                                    return 3;
                                };
                                gmts.chat_to_permlevel(
                                    &format!("&d[{}: Banning user {}]", our_name, args[0]),
                                    -1,
                                    4,
                                )
                                .await;
                                gmts.kick_user_by_name(&args[0], reason).await;
                                settings::add_banlist(&args[0], &reason.clone());
                            }
                        } else {
                            return 2;
                        }
                    } else {
                        return 3;
                    };
                    0
                })
            }),
        );
        pre_gmts.register_command(
            "kick".to_string(),
            "(player) (reason)",
            "Kick a user from the game.",
            Box::new(move |gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 4 {
                            if args.len() < 1 {
                                return 1;
                            }
                            let their_id = if let Some(i) = gmts.get_id(args[0].to_string()).await {
                                i
                            } else {
                                return 1;
                            };
                            let their_p_level = match gmts.get_permission_level(their_id).await {
                                Some(x) => x,
                                None => {
                                    return 1;
                                }
                            };
                            if their_p_level > p {
                                gmts.chat_to_id(
                                    "Can't kick a user with higher permissions.",
                                    -1,
                                    sender,
                                )
                                .await;
                            } else {
                                let mut reason = &args[1..].join(" ");
                                let string = "Kicked by an operator.".to_string();
                                if reason.len() < 1 {
                                    reason = &string;
                                }
                                gmts.chat_to_id(&format!("&fKicking user {}", args[0]), -1, sender)
                                    .await;
                                let our_name = if let Some(n) = gmts.get_username(sender).await {
                                    n
                                } else {
                                    return 3;
                                };
                                gmts.chat_to_permlevel(
                                    &format!("&d[{}: Kicking user {}]", our_name, args[0]),
                                    -1,
                                    4,
                                )
                                .await;
                                gmts.kick_user_by_name(&args[0], reason).await;
                            }
                        } else {
                            return 2;
                        }
                    } else {
                        return 3;
                    };
                    0
                })
            }),
        );
        pre_gmts.register_command(
            "op".to_string(),
            "(player)",
            "Give a player operator status.",
            Box::new(move |gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 4 {
                            if args.len() < 1 {
                                return 1;
                            }
                            let their_id = match gmts.get_id(args[0].clone()).await {
                                Some(x) => x,
                                None => {
                                    return 3;
                                }
                            };
                            let their_p_level = match gmts.get_permission_level(their_id).await {
                                Some(x) => x,
                                None => {
                                    return 1;
                                }
                            };
                            if their_p_level >= 4 {
                                gmts.chat_to_id(
                                    &format!("{} is already an op!", args[0]),
                                    -1,
                                    sender,
                                )
                                .await;
                            } else {
                                let our_name = if let Some(n) = gmts.get_username(sender).await {
                                    n
                                } else {
                                    return 3;
                                };
                                gmts.set_permission_level(their_id, 4).await;
                                gmts.message_to_id(
                                    PlayerCommand::RawPacket {
                                        bytes: vec![0x0f, 0x64],
                                    },
                                    their_id,
                                )
                                .await;
                                gmts.chat_to_permlevel(
                                    &format!("&d[{}: Opping {}]", our_name, args[0]),
                                    -1,
                                    4,
                                )
                                .await;
                                gmts.chat_to_id("&eYou are now op!", -1, their_id).await;
                                settings::add_op(&args[0]);
                            }
                        } else {
                            return 2;
                        }
                    } else {
                        return 3;
                    };
                    0
                })
            }),
        );
        pre_gmts.register_command(
            "deop".to_string(),
            "(player)",
            "Remove a player's operator status.",
            Box::new(move |gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 4 {
                            if args.len() < 1 {
                                return 1;
                            }
                            let their_id = match gmts.get_id(args[0].clone()).await {
                                Some(x) => x,
                                None => {
                                    return 3;
                                }
                            };
                            let their_p_level = match gmts.get_permission_level(their_id).await {
                                Some(x) => x,
                                None => {
                                    return 1;
                                }
                            };
                            if their_p_level > p {
                                gmts.chat_to_id(
                                    &format!("{} has higher permissions!", args[0]),
                                    -1,
                                    sender,
                                )
                                .await;
                                return 0;
                            }
                            if their_p_level <= 1 {
                                gmts.chat_to_id(
                                    &format!("{} is already not an op!", args[0]),
                                    -1,
                                    sender,
                                )
                                .await;
                            } else {
                                if let None = gmts.set_permission_level(their_id, 1).await {
                                    return 3;
                                }
                                let our_name = if let Some(n) = gmts.get_username(sender).await {
                                    n
                                } else {
                                    return 3;
                                };
                                gmts.message_to_id(
                                    PlayerCommand::RawPacket {
                                        bytes: vec![0x0f, 0x64],
                                    },
                                    their_id,
                                )
                                .await;
                                gmts.chat_to_permlevel(
                                    &format!("&d[{}: De-opping {}]", our_name, args[0]),
                                    -1,
                                    4,
                                )
                                .await;
                                gmts.chat_to_id("&eYou are no longer op!", -1, their_id)
                                    .await;
                                settings::remove_op(&args[0]);
                            }
                        } else {
                            return 2;
                        }
                    } else {
                        return 3;
                    };
                    0
                })
            }),
        );
        pre_gmts.register_setblock_hook(Box::new(|gmts: CMDGMTS, block, sender_id| {
            Box::pin(async move {
                if let Some(p) = gmts.get_permission_level(sender_id as i8).await {
                    if p < 4 {
                        let x = gmts.get_value("Coreutils_SpawnPosition").await?;
                        let spawn_pos = x.val.downcast_ref::<PlayerPosition>()?;
                        let spawn_pos = spawn_pos.clone();
                        let distance = spawn_pos.distance_to(block.position.clone());
                        if distance as u64 > CONFIGURATION.spawn_protection_radius {
                            return Some((block, sender_id));
                        } else {
                            return None;
                        }
                    } else {
                        return Some((block, sender_id));
                    }
                } else {
                    return Some((block, sender_id));
                }
            })
        }));
    }
}
