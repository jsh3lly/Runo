/* Netcode of the game
 * - For client-server communications, the program uses packets sent over TCP. The packets are
 * essentially PACKET_SIZE sized buffers which are either RequestPacket (client -> server) or
 * ResponsePacket (server -> client) types which are seralized/deserialized on respective ends
 *
 * - For intra-server communications, such serialization/deserialization isn't necessary, hence I
 * simply use Tokio channels for inter-thread communications
 */

use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering::SeqCst;
use std::thread;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};
use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize, serialize_into};
use RequestPacket::*;
use tokio::sync::broadcast;

use crate::card::{Card, Deck, Hand};
use bunt;

const MAX_PLAYERS_LIMIT : u8 = 10;
const PACKET_SIZE : usize = 1024;

#[derive(Serialize, Deserialize)]
enum RequestPacket {
    HELLO,
    // INITIALIZE,
}

#[derive(Serialize, Deserialize)]
enum ResponsePacket {
    ASSIGN_ID {id : u8},
    START_GAME {hand : Hand},
    ERROR {msg : String},
    // INITIALIZE_RESPONSE {hand: Vec<Card>},
}

// For now, deck is of size 1 (1 deck to play with)
struct GameState {
    deck: Deck,
    curr_player_count : u8,
    phase: Phase
}

#[derive(PartialEq, Clone)]
enum Phase {
    WAITING,        // Waiting for players to join.
    INGAME,         // The game has started
    GAMEOVER,       // Gane is over
}

#[derive(Clone, Debug, PartialEq)]
enum ServerBroadcastPacket {
    START_GAME,
}

impl GameState {
    fn new() -> GameState {
        GameState { deck: (Deck::new()), curr_player_count: 0, phase:Phase::WAITING }
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
    let (tx, _) = broadcast::channel::<ServerBroadcastPacket>(32);

    /*
     * Main server loop:
     * - Logs events from self (server) and clients
     * - Admin user can also run commands
     */
    tokio::spawn({
        let shared_state = shared_state.clone();
        let tx = tx.clone();
        async move {
            let mut line;
            loop {
                line = String::new();
                std::io::stdin().read_line(&mut line).unwrap();
                match line.trim() {
                    "start" => {
                        {
                            let res = tx.send(ServerBroadcastPacket::START_GAME);
                            if res.is_err() {println!("broadcast err")}
                        }
                    },
                    _ => unreachable!()
                }
                bunt::println!("{$yellow}{}{/$}", line);
            }
        }
    });

    loop {
        let (mut stream, peer_addr) = listener.accept().await?;

        // let shared_state_clone = &shared_state;
        // for every new connection
        let shared_state = shared_state.clone();

        // let tx = tx.clone();
        let mut rx = tx.subscribe();
        tokio::spawn(async move {
            // waiting for initial HELLO packet
            let curr_client_id : u8;
            let mut buff = [0u8; PACKET_SIZE];
            match stream.read_exact(&mut buff).await {
                Err(_) => bunt::println!("{$red}Error receiving packet from new client!{/$}"),
                Ok(_) => bunt::println!("{$green}Received Connection request from new client{/$}"),
            }
            let req_packet : RequestPacket = deserialize(&buff).unwrap();
            match req_packet {
                RequestPacket::HELLO => {
                    let phase = shared_state.lock().unwrap().phase.clone();
                    buff = [0u8; PACKET_SIZE];
                    match phase {
                        Phase::WAITING => {

                            {
                                let mut state = shared_state.lock().unwrap();
                                state.increment_player();
                                curr_client_id = state.get_curr_player_count();
                            }

                            serialize_into(&mut buff[..], &ResponsePacket::ASSIGN_ID { id: curr_client_id }).unwrap();
                            match stream.write_all(&buff[..PACKET_SIZE]).await {
                                Ok(_) => bunt::println!("{$green}ID assigned to new client is {}{/$}", curr_client_id),
                                Err(_) => bunt::println!("{$red}Error sending packet to {}{/$}", curr_client_id),
                            };
                        },
                        Phase::INGAME => {
                            serialize_into(&mut buff[..], &ResponsePacket::ERROR{ msg: "Game has already started!".to_string()}).unwrap();
                            stream.write_all(&buff[..PACKET_SIZE]).await.unwrap();
                            bunt::println!("{$red}Responded with error, since game has already started!{/$}");
                            return;
                        },
                        Phase::GAMEOVER => {
                            serialize_into(&mut buff[..], &ResponsePacket::ERROR{ msg: "Game is over!".to_string()}).unwrap();
                            stream.write_all(&buff[..PACKET_SIZE]).await.unwrap();
                            bunt::println!("{$red}Responded with error, since game has already started!{/$}");
                            return;
                        },
                    }

                }
            }

            bunt::println!("{$yellow}Connection established: {} {/$}", peer_addr);

            // Blocks until game is started
            loop {
                match rx.try_recv() {
                    Ok(_) => {break;},
                    Err(_) => {
                        // No message available, wait a bit and try again
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
            {shared_state.lock().unwrap().phase = Phase::INGAME};
            // bunt::println!("{$magenta}Game Started! {/$}");


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
    // let client_id : u8;

    // Sending HELLO packet to get a client ID
    // let mut buf : [isize; 1024] = serialize(&ResponsePacket::HELLO).unwrap().into_boxed_slice();
    let mut buff = [0u8; PACKET_SIZE];
    serialize_into(&mut buff[..], &RequestPacket::HELLO).unwrap();
    stream.write_all(&buff[..PACKET_SIZE]).await?;
    stream.flush().await?;
    println!("hello");
    buff = [0u8; PACKET_SIZE];
    stream.read_exact(&mut buff).await?;
    match deserialize(&buff).unwrap() {
        ResponsePacket::ASSIGN_ID { id } => {
            println!("Got ID: {}, wait until game admin starts the game", id);
            // client_id = id;
        },
        ResponsePacket::ERROR { msg } => {bunt::println!("{$red}{}{/$}", msg); return Ok(());},
        _ => panic!("Wrong packet received. This was not supposed to happen"),
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
