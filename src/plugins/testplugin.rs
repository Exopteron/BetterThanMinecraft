/* use crate::game::*;
use crate::classic::*;
use tokio::io::AsyncWriteExt;
pub struct TestPlugin {}
use std::sync::Arc;
impl crate::game::Plugin for TestPlugin {
    fn initialize(pre_gmts: &mut PreGMTS) {
        pre_gmts.cpe_required(false);
        pre_gmts.register_extension("CustomBlocks", 1, false);
        pre_gmts.register_early_onconnect_hook(Box::new(|gmts, stream, id| {
            Box::pin(async move {
                use tokio::io::AsyncReadExt;
                let supported_extensions = if let Some(x) = gmts.get_supported_extensions(id).await {
                    x
                } else {
                    return Some(());
                };
                if supported_extensions.get("CustomBlocks").is_some() && supported_extensions.get("CustomBlocks").unwrap().version == 1 {
                    let mut stream = stream.lock().await;
                    let mut packet = vec![];
                    packet.push(0x13);
                    packet.push(0x01);
                    stream.write(&packet).await.ok()?;
                    let mut recv_packet = [0; 2];
                    stream.read_exact(&mut recv_packet).await.ok()?;
                    if recv_packet[0] != 0x13 {
                        return None;
                    }
                    let their_level = recv_packet[1];
                    let mutual_level = std::cmp::min(1, their_level);
                    log::info!("Mutual level: {}", mutual_level);
                    log::info!("WHAT'S UP, ID {}", id);
                    return Some(());
                } 
/*                 if supported_extensions.get("LongerMessages").is_some() && supported_extensions.get("LongerMessages").unwrap().version == 1 {
                    let username = gmts.get_username(id).await?;
                    gmts.new_value(format!("{}_LMChatbox", username), GMTSElement { val: Arc::new(Box::new(String::new())) });
                } */
                return Some(());
            })
        }));
/*         pre_gmts.register_packet_hook(0x0d, Box::new(|gmts, stream, packet_id, sender_id| {
            Box::pin(async move {
                let mut stream = stream.lock().await;
                use tokio::io::AsyncReadExt;
                if let crate::classic::Packet::MessageC { message, unused } = ClassicPacketReader::read_packet_reader(&mut Box::pin(*stream)).await.ok()? {
                    let our_username = gmts.get_username(sender_id as i8).await?;
                    let our_id = sender_id;
                    if let Some(x) = gmts.get_supported_extensions(sender_id).await {
                        if x.get("LongerMessages").is_some() && x.get("LongerMessages").unwrap().version == 1 {
                            let mut chatbox = gmts.get_value(format!("{}_LMChatbox", our_username)).await?.val.downcast_ref::<String>()?;
                            chatbox.push_str(&message);
                            if unused == 0 {
                                gmts.set_value(format!("{}_LMChatbox", our_username), GMTSElement { val: Arc::new(Box::new(String::new())) });
                                let message = chatbox;
                                if message.starts_with("/") {
                                    gmts.execute_command(sender_id as i8, message.to_string()).await;
                                  } else {
                                    let mut prefix = format!("<{}> ", our_username);
                                    prefix.push_str(&message);
                                    let message = prefix;
                                    let message = message.as_bytes().to_vec();
                                    let message = message.chunks(64).collect::<Vec<&[u8]>>();
                                    let mut msg2 = vec![];
                                    for m in message {
                                      msg2.push(String::from_utf8_lossy(&m).to_string());
                                    }
                                    let m = msg2.remove(0);
                                    gmts.chat_broadcast(&m, (our_id as u8) as i8).await;
                                    for m in msg2 {
                                    gmts.chat_broadcast(&format!("> {}", m), (our_id as u8) as i8).await;
                                    }
                                  }
                                  return Some(());
                            } else {
                                gmts.set_value(format!("{}_LMChatbox", our_username), GMTSElement { val: Arc::new(Box::new(chatbox)) });
                            }
                        } 
                    };
                    if message.starts_with("/") {
                        gmts.execute_command(sender_id as i8, message).await;
                      } else {
                        let mut prefix = format!("<{}> ", our_username);
                        prefix.push_str(&message);
                        let message = prefix;
                        let message = message.as_bytes().to_vec();
                        let message = message.chunks(64).collect::<Vec<&[u8]>>();
                        let mut msg2 = vec![];
                        for m in message {
                          msg2.push(String::from_utf8_lossy(&m).to_string());
                        }
                        let m = msg2.remove(0);
                        gmts.chat_broadcast(&m, (our_id as u8) as i8).await;
                        for m in msg2 {
                        gmts.chat_broadcast(&format!("> {}", m), (our_id as u8) as i8).await;
                        }
                      }
                      return Some(());
                    //if unused == 0 {
                    //}
                }
                log::info!("Chat packet");
                Some(())
            })
        })); */
/*         pre_gmts.register_command("ping".to_string(), |gmts: CMDGMTS, args, sender| {
            Box::pin(async move {
                // do really cool things
                gmts.chat_to_id("Pong!", -1, sender).await;
                0
            })
        }); */

/*         pre_gmts.register_command("broadcast".to_string(), |gmts: CMDGMTS, args: Vec<String>, sender| {
            Box::pin(async move {
                // do really cool things
                gmts.chat_broadcast(&format!("&f[Broadcast] {}", args.join(" ")), -1)
                    .await;
                0
            })
        }); */
        pre_gmts.register_value("block_break_enabled", GMTSElement { val: Arc::new(Box::new(true)) }).unwrap();
        static X: usize = 42069;
        pre_gmts.register_command("msg".to_string(), "(user) (message)", "Message a user", Box::new(|gmts: CMDGMTS, args, sender| {
            Box::pin(async move {
                log::info!("X: {}", X);
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
                let msg = &format!("&8[&cme &8-> &c{}&8]&7 {}", args[0].clone(), args[1..].join(" "));
                gmts.chat_to_id(msg, -1, sender).await;
                let msg = &format!("&8[&c{} &8-> &cme&8]&7 {}", our_name, args[1..].join(" "));
                // do really cool things
                gmts.chat_to_username(&msg, -1, args[0].clone())
                    .await;
                0
            })
        }));
/*          pre_gmts.register_command("blockenabled".to_string(), "", "", Box::new(move |gmts: CMDGMTS, args, sender| {
            Box::pin(async move {
                let perm_level = if let Some(p) = gmts.get_permission_level(sender).await {
                    p
                } else {
                    return 3;
                };
                if perm_level < 4 {
                    return 2;
                }
                let current = if let Some(x) = gmts.get_value("block_break_enabled").await {
                    if let Some(val) = x.val.downcast_ref::<bool>() {
                        let x = val.clone();
                        x
                    } else {
                        return 3;
                    }
                } else {
                    return 3;
                };
                let current = current ^ true;
                if current {
                    gmts.chat_broadcast("&8[&cServer&8]&7 Placing blocks is now enabled.", -1).await;
                } else {
                    gmts.chat_broadcast("&8[&cServer&8]&7 Placing blocks is now disabled.", -1).await;
                }
                gmts.set_value("block_break_enabled", GMTSElement { val: Arc::new(Box::new(current))}).await;
                0
            })
        })); 
        pre_gmts.register_pmta_hook(Box::new(|gmts: CMDGMTS, command| {
            Box::pin(async move {
                if let PlayerCommand::Message { id, message } = &command {
                    log::info!("Very cool: {:?}", message);
                }
                command
            })
        })); */
/*         pre_gmts.register_setblock_hook(Box::new(|gmts: CMDGMTS, block, sender_id| {
            Box::pin(async move {
                let block_break = if let Some(x) = gmts.get_value("block_break_enabled").await {
                    let val = x.val;
                    if let Some(val) = val.downcast_ref::<bool>() {
                        let x = val.clone();
                        x
                    } else {
                        return None;
                    }
                } else {
                    return None;
                };
                let perm_level = if let Some(x) = gmts.get_permission_level(sender_id as i8).await {
                    x
                } else {
                    return None;
                };
                if perm_level < 4 && !block_break {
                    gmts.chat_to_id("Block interaction is currently disabled.", -1, sender_id as i8).await;
                    return None;
                }
                log::info!("Block {:?} placed by sender_id {}", block, sender_id);
                Some((block, sender_id))
            })
        })); */
    }
}
 */