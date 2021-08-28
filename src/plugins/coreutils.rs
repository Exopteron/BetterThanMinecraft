use crate::classic::*;
use crate::settings;
use crate::game::*;
pub struct CoreUtils {}
use std::sync::Arc;
/*

Command plan:
/ban - ban user
/unban - unban user
/op - op user
/deop - deop user
/msg - msg user
/r - reply to msg
/list - player list

*/
impl crate::game::Plugin for CoreUtils {
    fn initialize(pre_gmts: &mut PreGMTS) {
        pre_gmts.register_command(
            "permlevel".to_string(),
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
            "me".to_string(),
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
            "kick".to_string(),
            Box::new(move |gmts: CMDGMTS, args, sender| {
                Box::pin(async move {
                    if let Some(p) = gmts.get_permission_level(sender).await {
                        if p >= 4 {
                            if args.len() < 1 {
                                return 1;
                            }
                            gmts.chat_to_id(&format!("&fKicking user {}", args[0]), -1, sender)
                                .await;
                            gmts.kick_user_by_name(&args[0], &args[1..].join(" ")).await;
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
