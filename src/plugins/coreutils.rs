use crate::classic::*;
use crate::settings;
use crate::game::*;
pub struct CoreUtils {}
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
        pre_gmts.register_command(
            "help".to_string(),
            "",
            "Get command help",
            Box::new(move |gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 1 {
                            log::info!("hi!");
                            let all_cmds = gmts.get_commands_list().await;
                            gmts.chat_to_id("&fHelp:", -1, sender).await;
                            for (name, data) in all_cmds {
                                let message = format!("&c/{} {} &f- &7{}", name, data.args, data.desc).as_bytes().to_vec();
                                let message = message.chunks(60).collect::<Vec<&[u8]>>();
                                for message in message {
                                    gmts.chat_to_id(&format!("&7{}",String::from_utf8_lossy(message).to_string()), -1, sender).await;
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
                            gmts.chat_to_permlevel(&format!("&d[{}: Stopping the server...]", our_name), -1, 4).await;
                            if let None = gmts.stop_server().await {
                                gmts.chat_to_permlevel(&format!("&d[{}: Error stopping the server.]", our_name), -1, 4).await;
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
                            gmts.chat_to_permlevel(&format!("&d[{}: Forcing save..]", our_name), -1, 4).await;
                            if let None = gmts.save_world().await {
                                gmts.chat_to_permlevel(&format!("&d[{}: Error saving the world.]", our_name), -1, 4).await;
                                return 3;
                            }
                            gmts.chat_to_permlevel(&format!("&d[{}: Save complete.]", our_name), -1, 4).await;
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
                            let reason = &args[1..].join(" ");
                            gmts.chat_to_id(&format!("&fKicking user {}", args[0]), -1, sender).await;
                            let our_name = if let Some(n) = gmts.get_username(sender).await {
                                n
                            } else {
                                return 3;
                            };
                            gmts.chat_to_permlevel(&format!("&d[{}: Kicking user {}]", our_name, args[0]), -1, 4).await;
                            gmts.kick_user_by_name(&args[0], reason).await;
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
                                gmts.chat_to_id(&format!("{} is already an op!", args[0]), -1, sender).await;
                            } else {
                                gmts.set_permission_level(their_id, 4).await;
                                gmts.chat_to_id(&format!("Opping user {}", args[0]), -1, sender).await;
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
                            if their_p_level <= 1 {
                                gmts.chat_to_id(&format!("{} is already not an op!", args[0]), -1, sender).await;
                            } else {
                                gmts.set_permission_level(their_id, 1).await;
                                gmts.chat_to_id(&format!("De-opping user {}", args[0]), -1, sender).await;
                                gmts.chat_to_id("&eYou are no longer op!", -1, their_id).await;
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
    }
}
