/* Netcode of the game
 * - For client-server communications, the program uses packets sent over TCP. The packets are
 * essentially PACKET_SIZE sized buffers which are either RequestPacket (client -> server) or
 * ResponsePacket (server -> client) types which are seralized/deserialized on respective ends
 *
 * - For intra-server communications, such serialization/deserialization isn't necessary, hence I
 * simply use Tokio channels for inter-thread communications
 */

use std::io::BufRead;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering::SeqCst;
use std::{thread, io};
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};
use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize, serialize_into};
use RequestPacket::*;
use tokio::sync::broadcast;

use crate::card::{Card, Deck, Hand};
use bunt::{self, print, println};

const MAX_PLAYERS_LIMIT : u8 = 10;
const PACKET_SIZE : usize = 1024;

macro_rules! cls {
    () => {
        print!("\x1B[2J\x1b[1;1H");
    }
}

macro_rules! nice_panic {
    () => {
        panic!("Wrong packet received. This was not supposed to happen")
    };
}


#[derive(Serialize, Deserialize)]
enum RequestPacket {
    HELLO,
    DO_TURN {card: Card},
    ACKNOWLEDGE_UPDATE,
}

#[derive(Serialize, Deserialize)]
enum ResponsePacket {
    ASSIGN_ID {id : usize},
    START_GAME {hand : Hand},
    GIVE_UPDATE {curr_player_turn: usize, topmost_card: Card}, //TODO: might wanna add card_debt : Option<usize> 
    ERROR {msg : String},
    // INITIALIZE_RESPONSE {hand: Vec<Card>},
}

// For now, deck is of size 1 (1 deck to play with)
struct GameState {
    deck: Deck,
    players: Vec<usize>,
    phase: Phase,
    top_card: Card,
    curr_player_turn_index : usize,
}

struct ClientInfo {
    id : usize,
    hand: Hand,
    // name : String,
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
        let mut deck = Deck::new();
        let top_card = deck.pop_random_card();
        GameState { deck,
            players: vec![],
            phase: Phase::WAITING,
            top_card,
            curr_player_turn_index:0 }
    }
    fn add_new_player(&mut self) {
        match self.players.last() {
            Some(x) => self.players.push(x+1),
            None => self.players.push(1),
        }
    }
    fn players_len(&self) -> usize {
        self.players.len()
    }
    fn end_turn(&mut self) {
        // IMP: this might be a bug
        if self.curr_player_turn_index < self.players_len() - 1 {
            self.curr_player_turn_index += 1;
        }
        else {
            self.curr_player_turn_index = 0; // wrap around
        }
    }
    fn get_curr_player_turn(&self) -> usize {
        self.players[self.curr_player_turn_index]
    }
}

pub async fn run_server(port : u32) -> Result<(), Box<dyn std::error::Error>> {
    cls!();
    // let local_ip = std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);
    // let bind_address = SocketAddr::new(local_ip, port.try_into().unwrap());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
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
        let server_commands = vec!["start"]; //TODO: this isn't the best way to do this
        async move {
            let mut line;
            loop {
                line = String::new();
                std::io::stdin().read_line(&mut line).unwrap();

                match line.trim() {
                    "start" => {
                        if shared_state.lock().unwrap().phase == Phase::WAITING {
                            let curr_player_count = shared_state.lock().unwrap().players_len();
                            if  curr_player_count < 2 {
                                bunt::println!("{$red}Game can only be started with 2 or more players,
                                               only {} player(s) connected for now{/$}", curr_player_count);
                                continue;
                            }
                            let res = tx.send(ServerBroadcastPacket::START_GAME);
                            if res.is_err() {println!("broadcast err")}
                            cls!();
                            bunt::println!("{$magenta}Game Started! {/$}");
                        }
                        else {bunt::println!("{$red}Game has already started!{/$}")}
                    },
                    _ => {bunt::println!("{$red}Invalid command: Valid commands are: {:?}{/$}", server_commands)}
                }
                // bunt::println!("{$yellow}{}{/$}", line);
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
            // let client_info : ClientInfo;
            let client_id;
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
                                state.add_new_player();
                                client_id = state.players_len();
                            }

                            serialize_into(&mut buff[..], &ResponsePacket::ASSIGN_ID { id: client_id}).unwrap();
                            match stream.write_all(&buff[..PACKET_SIZE]).await {
                                Ok(_) => bunt::println!("{$green}ID assigned to new client is {}{/$}", client_id),
                                Err(_) => bunt::println!("{$red}Error sending packet to {}{/$}", client_id),
                            };
                        },
                        Phase::INGAME => {
                            serialize_into(&mut buff[..],
                                           &ResponsePacket::ERROR{ msg: "Cannot connect: Game has already started!".to_string()}).unwrap();
                            stream.write_all(&buff[..PACKET_SIZE]).await.unwrap();
                            bunt::println!("{$red}Responded with error, since game has already started!{/$}");
                            return;
                        },
                        Phase::GAMEOVER => {
                            serialize_into(&mut buff[..],
                                           &ResponsePacket::ERROR{ msg: "Cannot connect: Game is over!".to_string()}).unwrap();
                            stream.write_all(&buff[..PACKET_SIZE]).await.unwrap();
                            bunt::println!("{$red}Rejected joinging, since game has already started!{/$}");
                            return;
                        },
                    }

                }
                _ => nice_panic!()
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


            let hand : Hand = Hand::new(7, &mut shared_state.lock().unwrap().deck);
            //TODO: Handle case where deck runs out of cards

            // println!("{}", shared_state.lock().unwrap().deck.len());
            buff = [0u8; PACKET_SIZE];
            serialize_into(&mut buff[..], &ResponsePacket::START_GAME { hand }).unwrap();
            stream.write_all(&buff[..PACKET_SIZE]).await.unwrap();


            // Serving the client, game loop
            loop {
                {
                    let state = shared_state.lock().unwrap();
                    buff = [0u8; PACKET_SIZE];
                    serialize_into(&mut buff[..], &ResponsePacket::GIVE_UPDATE { curr_player_turn: state.get_curr_player_turn(),
                    topmost_card: state.top_card.clone() }).unwrap();
                }
                stream.write_all(&buff[..PACKET_SIZE]).await.unwrap();

                buff = [0u8; PACKET_SIZE];
                stream.read_exact(&mut buff).await;
                match deserialize::<RequestPacket>(&buff).unwrap() {
                    RequestPacket::DO_TURN { card } => {
                        shared_state.lock().unwrap().end_turn();
                        println!("chosen card: {}", card);
                    },
                    // RequestPacket::QUIT => {}
                    RequestPacket::ACKNOWLEDGE_UPDATE => {thread::sleep(Duration::from_millis(300))},
                    _ => nice_panic!(),
                }

                // if buff.is_empty() {continue;}
            }

            // println!("Connection closed: {}", peer_addr);
        });
    }
}

pub async fn run_client(port : u32) -> Result<(), Box<dyn std::error::Error>> {
    cls!();
    // let local_ip = std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);
    // let bind_address = SocketAddr::new(local_ip, port.try_into().unwrap());
    let mut stream = TcpStream::connect(format!("0.0.0.0:{}", port)).await?;
    let mut client_info : ClientInfo;
    let client_id : usize;

    // Sending HELLO packet to get a client ID
    // let mut buf : [isize; 1024] = serialize(&ResponsePacket::HELLO).unwrap().into_boxed_slice();
    let mut buff = [0u8; PACKET_SIZE];
    serialize_into(&mut buff[..], &RequestPacket::HELLO).unwrap();
    stream.write_all(&buff[..PACKET_SIZE]).await?;
    stream.flush().await?;
    buff = [0u8; PACKET_SIZE];
    stream.read_exact(&mut buff).await?;
    match deserialize(&buff).unwrap() {
        ResponsePacket::ASSIGN_ID { id } => {
            bunt::println!("{$green}You are Player #{}, wait until game admin starts the game{/$}", id);
            client_id = id;
        },
        ResponsePacket::ERROR { msg } => {bunt::println!("{$red}{}{/$}", msg); return Ok(());},
        _ => nice_panic!(),
    }
    buff = [0u8; PACKET_SIZE];
    stream.read_exact(&mut buff).await?;
    match deserialize(&buff).unwrap() {
        ResponsePacket::START_GAME { hand } => {client_info = ClientInfo {id:client_id, hand}},
        _ => nice_panic!()
    }
    cls!();
    bunt::println!("{$green}The game has started!{/$}");

    // Game loop
    loop {
        cls!();
        buff = [0u8; PACKET_SIZE];
        stream.read_exact(&mut buff).await?;

        // Listen for updates from the server
        match deserialize(&buff).unwrap() {
            ResponsePacket::GIVE_UPDATE { curr_player_turn, topmost_card } => {
                if curr_player_turn == client_info.id {
                    bunt::println!("{$yellow}It is your turn!{/$}");
                }
                println!();
                bunt::println!("{$yellow}You are Player #{}{/$}", client_id);
                bunt::println!("{$yellow}Topmost card: {}{/$}", topmost_card);
                bunt::println!("{$yellow}Current turn's player: {}{/$}", curr_player_turn);
                println!("{}", client_info.hand);
                if curr_player_turn == client_info.id {
                    println!("Write the card number you wanna play: ");
                    let input = io::stdin().lock().lines().next().unwrap().unwrap();
                    let index = input.parse::<usize>().unwrap();
                    buff = [0u8; PACKET_SIZE];
                    serialize_into(&mut buff[..], &RequestPacket::DO_TURN { card: client_info.hand.pop_at(index) }).unwrap();
                    stream.write_all(&buff[..PACKET_SIZE]).await.unwrap();
                }
                else {
                    buff = [0u8; PACKET_SIZE];
                    serialize_into(&mut buff[..], &RequestPacket::ACKNOWLEDGE_UPDATE).unwrap();
                    stream.write_all(&buff[..PACKET_SIZE]).await.unwrap();
                }
            }
            _ => nice_panic!(),
        }
    }
}
