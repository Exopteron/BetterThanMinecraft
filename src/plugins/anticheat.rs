use crate::classic::*;
use crate::game::*;
use crate::settings;
pub struct Anticheat {}
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
use tokio::time::{sleep, Duration};
fn random_server_salt() -> String {
    use rand::RngCore;
    let mut bytes = vec![0; 15];
    let mut rng = rand::rngs::OsRng::new().unwrap();
    rng.fill_bytes(&mut bytes);
    return base_62::encode(&bytes);
}
impl crate::game::Plugin for Anticheat {
    fn initialize(pre_gmts: &mut PreGMTS) {
        pre_gmts.register_packet_hook(
            0x08,
            Box::new(|gmts, stream, packet_id, sender_id| {
                Box::pin(async move {
                    let mut stream = stream.lock().await;
                    let username = gmts.get_username(sender_id as i8).await?;
                    if let crate::classic::Packet::PositionAndOrientationC { position, .. } =
                        ClassicPacketReader::read_packet_reader(&mut Box::pin(&mut *stream), &username)
                            .await
                            .ok()?
                    {
                        if !CONFIGURATION.anticheat.anti_speed_tp {
                            gmts.send_position_update(sender_id as i8, position).await;
                            return Some(());
                        }
                        if let Some(p) = gmts.get_permission_level(sender_id as i8).await {
                            if p < 4 {
                                let last_position = gmts.get_position(sender_id as i8).await?;
                                let distance = position.distance_to_plr(last_position);
                                if distance > 6.0 {
                                    let message = PlayerCommand::PlayerTeleport { position: last_position, id: -1 };
                                    gmts.message_to_id(message, sender_id).await?;
                                    return None;
                                }
                                gmts.send_position_update(sender_id as i8, position).await;
                                return Some(());
                            } else {
                                gmts.send_position_update(sender_id as i8, position).await;
                            }
                        } else {
                            return None;
                        }
                    }
                    Some(())
                })
            }),
        );

        pre_gmts.register_setblock_hook(Box::new(|gmts, block, sender_id| {
            Box::pin(async move {
                if let Some(p) = gmts.get_permission_level(sender_id as i8).await {
                    if p < 4 {
                        let last_position = gmts.get_position(sender_id as i8).await?;
                        let distance = last_position.distance_to(block.position.clone());
                        if distance > CONFIGURATION.anticheat.reach_distance {
                            return None;
                        }
                        return Some((block, sender_id));
                    } else {
                        return Some((block, sender_id));
                    }
                } else {
                    return None;
                }
            })
        }));
    }
}
