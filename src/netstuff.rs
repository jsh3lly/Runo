
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering::SeqCst;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};
use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize, serialize_into};
use ResponsePacket::{ASSIGN_ID};
use RequestPacket::*;

use crate::card::{Card, Deck};
const MAX_PLAYERS_LIMIT : u8 = 10;
const PACKET_SIZE : usize = 1024;

// struct Packet {
//     kind: PacketKinds,
//     content: String,
// }
//
// enum PacketKinds {
//     HELLO, BYE, MOVE
// }


#[derive(Serialize, Deserialize)]
enum RequestPacket {
    HELLO,
}


#[derive(Serialize, Deserialize)]
enum ResponsePacket {
    ASSIGN_ID {id : u8},
}

// For now, deck is of size 1 (1 deck to play with)
struct GameState {
    deck: Deck,
    curr_player_count : u8,
}

impl GameState {
    fn new() -> GameState {
        GameState { deck: (Deck::new()), curr_player_count: 0 }
    }
    fn increment_player(&mut self) {
        self.curr_player_count += 1;
    }
    fn get_curr_player_count(&self) -> u8 {
        self.curr_player_count
    }
}

pub async fn run_server(port : u32) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    // let curr_player_count = Arc::new(AtomicU8::new(0));
    let shared_state = Arc::new(Mutex::new(GameState::new()));

    // Main server loop
    loop {
        let (mut stream, peer_addr) = listener.accept().await?;

        let shared_state_clone = Arc::clone(&shared_state);
        // for every new connection
        tokio::spawn(async move {
            println!("Connection established: {}", peer_addr);

            // waiting for initial HELLO packet

            let curr_client_id : u8;
            let mut buff = [0u8; PACKET_SIZE];
            match stream.read_exact(&mut buff).await {
                Err(_) => println!("Error receiving packet!"),
                Ok(_) => println!("Received packet from new client"),
            }
            let req_packet : RequestPacket = deserialize(&buff).unwrap();
            match req_packet {
                RequestPacket::HELLO => {

                    // let mut state = shared_state.lock().unwrap();
                    // state.increment_player();
                    // curr_client_id = state.get_curr_player_count();
                    
                    curr_client_id = 0;
                    let mut buff = [0u8; PACKET_SIZE];
                    serialize_into(&mut buff[..], &ResponsePacket::ASSIGN_ID { id: curr_client_id }).unwrap();
                    match stream.write_all(&buff[..PACKET_SIZE]).await {
                        Ok(_) => println!("Packet sent. ID assigned to new client is {}", curr_client_id),
                        Err(_) => println!("Error sending packet to {}", curr_client_id),
                    };
                }
            }

            // Serving the client, loop
            // loop {
            //     if buff.is_empty() {continue;}
            // }

            // println!("Connection closed: {}", peer_addr);
        });
    }
}

pub async fn run_client(port : u32) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
    let client_id : u8;

    // Sending HELLO packet to get a client ID
    // let mut buf : [isize; 1024] = serialize(&ResponsePacket::HELLO).unwrap().into_boxed_slice();
    let mut buff = [0u8; PACKET_SIZE];
    serialize_into(&mut buff[..], &RequestPacket::HELLO).unwrap();
    stream.write_all(&buff[..PACKET_SIZE]).await?;
    stream.flush().await?;
    println!("hello");
    buff = [0u8; PACKET_SIZE];
    stream.read_exact(&mut buff).await?;
    let resp_packet : ResponsePacket = deserialize(&buff).unwrap();

    match resp_packet {
        ASSIGN_ID { id } => println!("Got ID: {}", id)
    }

    // for i in 1..=3 {
    //     let message = format!("Hello, packet {}!\n", i);
    //     stream.write_all(message.as_bytes()).await?;
    //
    //     let mut reader = BufReader::new(&mut stream);
    //     let mut buffer = String::new();
    //     reader.read_line(&mut buffer).await?;
    //
    //     println!("Received: {}", buffer.trim_end());
    // }

    Ok(())
}
