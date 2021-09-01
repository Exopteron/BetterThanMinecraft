use crate::game::*;
use crate::classic::*;
pub struct CPESupporter {}
use std::sync::Arc;
impl crate::game::Plugin for CPESupporter {
    fn initialize(pre_gmts: &mut PreGMTS) {
        pre_gmts.cpe_required(true);
        pre_gmts.register_extension("HeldBlock", 1, true);
        let extensions = pre_gmts.cpe_handler.extensions.clone();
        for extension in extensions {
            match extension {
                CPEExtension::CustomBlocks { enabled, support_level } => {
                    pre_gmts.register_value(
                        "CPE_CBSupportLevel",
                        GMTSElement {
                            val: Arc::new(Box::new(support_level)),
                        },
                    )
                    .unwrap();
                    if enabled {
                        pre_gmts.register_extension("CustomBlocks", 1, true);
                        pre_gmts.register_early_onconnect_hook(Box::new(|gmts, stream, v_token, id| {
                            Box::pin(async move {
                                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                                let supported_extensions = if let Some(x) = gmts.get_supported_extensions(id).await {
                                    x
                                } else {
                                    return Some(());
                                };
                                if supported_extensions.get("CustomBlocks").is_some() && supported_extensions.get("CustomBlocks").unwrap().version == 1 {
                                    let x = if let Some(l) = gmts.get_value("CPE_CBSupportLevel").await {
                                        l
                                    } else {
                                        log::error!("Verify name error!");
                                        return None;
                                    };
                                    let support_level = if let Some(l) = x.val.downcast_ref::<u8>() {
                                        l
                                    } else {
                                        log::error!("Verify name error!");
                                        return None;
                                    };
                                    let mut stream = stream.lock().await;
                                    let mut packet = vec![];
                                    packet.push(0x13);
                                    packet.push(*support_level);
                                    stream.write(&packet).await.ok()?;
                                    let mut recv_packet = [0; 2];
                                    stream.read_exact(&mut recv_packet).await.ok()?;
                                    if recv_packet[0] != 0x13 {
                                        return None;
                                    }
                                    let their_level = recv_packet[1];
                                    let mutual_level = std::cmp::min(1, their_level);
                                    return Some(());
                                }
                                return Some(());
                            })
                        }));
                    }
                }
                CPEExtension::HeldBlock { enabled } => {
                    if enabled {
                    }
                }
            }
        }
    }
}