use crate::game::*;
use crate::classic::*;
pub struct LongerMessagesCPE {}
use std::sync::Arc;
impl crate::game::Plugin for LongerMessagesCPE {
    fn initialize(pre_gmts: &mut PreGMTS) {
        pre_gmts.cpe_required(false);
        pre_gmts.register_extension("LongerMessages", 1, false);
        pre_gmts.register_ondisconnect_hook(Box::new(|gmts, id| {
            Box::pin(async move {
                return Some(());
                let supported_extensions = if let Some(x) = gmts.get_supported_extensions(id).await {
                    x
                } else {
                    return Some(());
                };
                if supported_extensions.get("LongerMessages").is_some() && supported_extensions.get("LongerMessages").unwrap().version == 1 {
                    let username = gmts.get_username(id).await?;
                    gmts.rem_value(&format!("{}_LMChatbox", username)).await;
                }
                return Some(());
            })
        }));
         pre_gmts.register_early_onconnect_hook(Box::new(|gmts, stream, id| {
            Box::pin(async move {
                let supported_extensions = if let Some(x) = gmts.get_supported_extensions(id).await {
                    x
                } else {
                    return Some(());
                };
                if supported_extensions.get("LongerMessages").is_some() && supported_extensions.get("LongerMessages").unwrap().version == 1 {
                    let username = gmts.get_username(id).await?;
                    gmts.new_value(&format!("{}_LMChatbox", username), GMTSElement { val: Arc::new(Box::new(String::new())) }).await;
                }
                return Some(());
            })
        }));
        pre_gmts.register_packet_hook(0x0d, Box::new(|gmts, stream, packet_id, sender_id| {
            Box::pin(async move {
                let mut stream = stream.lock().await;
                use tokio::io::AsyncReadExt;
                if let crate::classic::Packet::MessageC { message, unused } = ClassicPacketReader::read_packet_reader(&mut Box::pin(&mut *stream)).await.ok()? {
                    let our_username = gmts.get_username(sender_id as i8).await?;
                    let our_id = sender_id;
                    if let Some(x) = gmts.get_supported_extensions(sender_id).await {
                        if x.get("LongerMessages").is_some() && x.get("LongerMessages").unwrap().version == 1 {
                            let x = gmts.get_value(&format!("{}_LMChatbox", our_username)).await?;
                            let chatbox = x.val.downcast_ref::<String>()?;
                            let mut chatbox = chatbox.clone();
                            chatbox.push_str(&message);
                            if unused == 0 {
                                gmts.set_value(&format!("{}_LMChatbox", our_username), GMTSElement { val: Arc::new(Box::new(String::new())) }).await;
                                let message = chatbox;
                                if message.starts_with("/") {
                                    gmts.execute_command(sender_id as i8, message.to_string()).await;
                                  } else {
                                    let mut prefix = format!("<{}> ", our_username);
                                    prefix.push_str(&message);
                                    let message = prefix;
                                    let message = message.as_bytes().to_vec();
                                    let message = message.chunks(61).collect::<Vec<&[u8]>>();
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
                            } else {
                                gmts.set_value(&format!("{}_LMChatbox", our_username), GMTSElement { val: Arc::new(Box::new(chatbox)) }).await;
                            }
                            return Some(());
                        } 
                    };
                    log::info!("Got to here");
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
                }
                log::info!("Chat packet");
                Some(())
            })
        }));
    }
}
