use std::sync::Arc;
use crate::game::GMTS;
use async_tungstenite::tokio::TokioAdapter;
use tokio::io::{AsyncWriteExt};
use tokio::io::AsyncReadExt as TokioRead;
use futures::AsyncWriteExt as FuturesWrite;
use
{
	ws_stream_tungstenite :: { *                                         } ,
	futures               :: { AsyncReadExt, io::{ BufReader, copy_buf } } ,
	std                   :: { env, net::SocketAddr, io                  } ,
	log                   :: { *                                         } ,
	tokio                 :: { net::{ TcpListener, TcpStream }           } ,
	async_tungstenite     :: { accept_async     } ,
};

pub async fn main(gmts: Arc<GMTS>) {
    let socket = TcpListener::bind("0.0.0.0:35565").await.unwrap();
    loop {
        let gmts = gmts.clone();
        let (stream, _) = socket.accept().await.unwrap();
        tokio::spawn(async move {
            handle_connection(stream).await;
        });
    }
} 

async fn handle_connection(stream: TcpStream) {
    let s = accept_async(TokioAdapter::new(stream)).await.unwrap();
    let ws_stream = WsStream::new( s );
    let stream = TcpStream::connect("localhost:25510").await.unwrap();
    log::info!("Websocket");
    let (mut ws_rd, mut ws_wr) = ws_stream.split();
    let (mut s_rd, mut s_wr) = stream.into_split();
    tokio::spawn(async move {
        loop {
            let mut byte = [0; 1];
            ws_rd.read_exact(&mut byte).await;
            s_wr.write(&byte).await;
        }
    });
    loop {
        let mut byte = [0; 1];
        s_rd.read_exact(&mut byte).await;
        ws_wr.write(&byte).await;
    }
}