use std::{net::{TcpListener, TcpStream}, io, sync::{Arc, Mutex}, collections::VecDeque};
use rand::Rng;
use tokio::sync::broadcast;

use crate::{netcode::packets::{send_packet, ServerPacket, read_packet, ClientPacket, GameThreadBroadcastPacket}, card::{Deck, Hand}};
use crate::netcode::misc::Names;
use crate::card;

use card::Card;

const MAX_PLAYERS_LIMIT : u8 = 10;
macro_rules! cls {
    () => {
        print!("\x1B[2J\x1b[1;1H");
    }
}

macro_rules! server_received_unexpected_packet {
    () => {
        bunt::println!("{$red}Server received unexpected packet from client{/$}")
    };
}


macro_rules! client_received_unexpected_packet {
    () => {
        bunt::println!("{$red}Client received unexpected packet from server{/$}")
    };
}

#[derive(Debug, PartialEq, PartialOrd)]
enum GamePhase {
    Waiting,
    InGame,
    GameOver,
}

#[derive(Debug)]
enum Direction {
    Positive,
    Negative,
}

#[derive(Debug)]
struct ClientInfo {
    id: usize,
    stream: TcpStream,
    name: String,
    hand: Hand,
}

#[derive(Debug)]
struct GlobalGameData {
    names : Names,
    game_phase: GamePhase,
    curr_total_clients_num : usize,
    curr_client_id_turn: usize, /// Number between 0 and curr_clients_num (non inclusive).
    master_deck : Deck,
    direction: Direction,
    card_debt: usize,
    skip_debt: usize,
    stack: VecDeque<Card>,
    clients_info: Vec<ClientInfo>,
}

impl GlobalGameData {
    fn get_players_string(&self) -> String {
        let mut ret_string = String::new();
        for client_idx in 0..self.clients_info.len() {
            if client_idx == self.curr_client_id_turn {
                ret_string += "* ";
            }
            ret_string += &self.clients_info[client_idx].name;
            ret_string += "\n"
        }
        ret_string
    }

    /// Goes to the next player after accounting for skip_debt and direction
    fn next_player(&mut self) {
        let lhs = self.curr_client_id_turn as isize;
        let rhs : isize;
        match self.direction {
            Direction::Positive => rhs = 1 + self.skip_debt as isize,
            Direction::Negative => rhs = - (1 + self.skip_debt as isize),
        }
        let next_player = (lhs + rhs) % self.curr_total_clients_num as isize;
        // if next_player < 0 || next_player >= self.curr_total_clients_num as isize {
        //     next_player = next_player % self.curr_total_clients_num as isize;
        // }
        self.curr_client_id_turn = next_player as usize;
    }
}

pub async fn run_server(port : u32, server_is_open : bool) -> Result<(), Box<dyn std::error::Error>> {
    bunt::println!("{$green}The server has been started{/$}");
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    let mut rng = rand::thread_rng();
    let join_code = rng.gen_range(100_000..999_999);
    if !server_is_open {
        bunt::println!("{$green}Joining code is: {} {/$}", join_code);
    }

    let mut deck = Deck::new();
    let stack_card = deck.pop_random_card();
    let shared_global_game_data = Arc::new(Mutex::new(GlobalGameData {
        names: Names::new(),
        game_phase: GamePhase::Waiting,
        curr_total_clients_num: 0, /// Total number of connected clients
        curr_client_id_turn:0, /// Number between 0 and curr_clients_num (non inclusive).
        master_deck: deck, /// The main deck from where cards are taken to form hands
        direction: Direction::Positive, // Two directions in which the game goes. Changes when reverse card is used
        card_debt: 0,
        skip_debt: 0,
        stack: VecDeque::from(vec![stack_card]),
        clients_info: vec![],
    }));

    let (tx, _) = broadcast::channel::<GameThreadBroadcastPacket>(32);

    /// IMP: This function holds shared_state for a long time
    async fn game_thread(shared_state: Arc<Mutex<GlobalGameData>>) {
        let mut shrared_state_held = shared_state.lock().unwrap();
        dbg!(&shrared_state_held.game_phase);
        shrared_state_held.game_phase = GamePhase::InGame;
        loop {
            // provide updates to players
            for idx in 0..shrared_state_held.clients_info.len() {
                let mut msg = "Players: \n".to_string() + &shrared_state_held.get_players_string() + "\n";
                msg += &format!("Topmost card: {}", shrared_state_held.stack.get(0).unwrap()).to_string();
                msg += "\n";
                let hand_copy = shrared_state_held.clients_info[idx].hand.clone();
                let is_my_chance;
                is_my_chance = idx == shrared_state_held.curr_client_id_turn;
                send_packet(&mut shrared_state_held.clients_info[idx].stream,
                            ServerPacket::SendMsgUpdate { msg, hand: hand_copy, is_my_chance });
            }
            let curr_client_id = shrared_state_held.curr_client_id_turn;
            let client_send_move_packet = read_packet::<ClientPacket>(&mut shrared_state_held.clients_info[curr_client_id].stream);
            match client_send_move_packet {
                ClientPacket::SendMove { card_idx } => {
                    let card = shrared_state_held.clients_info[curr_client_id].hand.get_at(card_idx);
                    println!("{}", card);
                    shrared_state_held.next_player();
                }
                _ => server_received_unexpected_packet!(),
            }
        }
        // loop {
        //     
        // }
        // println!("Hello world");
        // println!("{}", shared_state.lock().unwrap().get_players_string());
        // shared_state.lock().unwrap().next_player();
        // println!("{}", shared_state.lock().unwrap().get_players_string());
        // shared_state.lock().unwrap().next_player();
        // println!("{}", shared_state.lock().unwrap().get_players_string());
        
        // send_packet(&mut shared_state.lock().unwrap().clients_info[0].stream, ServerPacket::SendMsg { msg: Some(shared_state.lock().unwrap().get_players_string()) });
    }

    /*
     * server commands thread: Executes commands sent to the server.
     */
    tokio::spawn({
        let shared_state = shared_global_game_data.clone();
        // let tx = tx.clone();
        async move {
            let mut input_line;
            let possible_commands = vec!["start", "clients_info"].iter().map(|elem| elem.to_string()).collect::<Vec<String>>();
            loop {
                input_line = String::new();
                std::io::stdin().read_line(&mut input_line).unwrap();
                input_line = input_line.trim().to_lowercase();
                if !possible_commands.contains(&input_line) {
                    bunt::println!("{$red}Unrecongized command. Valid commnads are: {:?} {/$}", possible_commands);
                }
                else if input_line == "clients_info" {
                    let shared_state_locked = shared_state.lock().unwrap();
                    dbg!(shared_state_locked); //FIXME: There is a warning!
                }
                else if input_line == "start" {
                    if shared_state.lock().unwrap().game_phase != GamePhase::Waiting {
                        bunt::println!("{$red}Game cannot be started if it already has started{/$}");
                    }
                    else if shared_state.lock().unwrap().curr_total_clients_num < 2 {
                        bunt::println!("{$red}Game cannot be started if number of players less than 2{/$}");
                    }
                    else {
                        // tx.send(GameThreadBroadcastPacket::StartGame).unwrap();
                        bunt::println!("{$magenta}Game Started!{/$}");
                        tokio::spawn(game_thread(shared_state.clone())).await;
                    }
                }
            }
        }
    });

    /* 
     * Client serving server thread: A server thread (per client) to serve the client. All these
     * threads "talk" with the 
     */
    loop {
        let (mut stream, peer_addr) = listener.accept()?;
        let shared_state = shared_global_game_data.clone();
        let mut rx = tx.subscribe();

        // for every new connection
        tokio::spawn(async move {
            match server_is_open {
                true => {
                    bunt::println!("{$green}A client connected!{/$}");
                    send_packet(&mut stream, ServerPacket::AuthRequest {required: false});
                }
                false => {
                    // try authenticating the client until it is authenticated
                    bunt::println!("{$yellow}A Client is attemtping to join...{/$}");
                    //TODO: Consider adding "number of tries" for the client to join in
                    let mut is_client_authenticated = false;
                    while !is_client_authenticated {
                        send_packet(&mut stream, ServerPacket::AuthRequest {required: true});
                        match read_packet::<ClientPacket>(&mut stream) {
                            ClientPacket::AuthResponse { join_code: code } => {
                                if code == join_code {
                                    bunt::println!("{$green}A client was successfully authentiated and connected!{/$}");
                                    is_client_authenticated = true;
                                    send_packet(&mut stream, ServerPacket::AuthAcknowledged)
                                }
                            }
                            _ => server_received_unexpected_packet!()
                        }
                    }
                }
            }

        // ==== Setting Client Name ====
        send_packet(&mut stream, ServerPacket::AskPreferredName);
        // Wait for client to send thier name
        match read_packet::<ClientPacket>(&mut stream) {
            ClientPacket::SendPreferredName { optional_client_name} => {
                let (ret_name, ret_msg);
                match optional_client_name {
                    Some(name) => {
                        let specific_name_result = shared_state.lock().unwrap().names.get_specific_name(name);
                        match specific_name_result {
                            Ok(name) => {ret_name = name; ret_msg = None;}
                            Err(()) => {
                                ret_name = shared_state.lock().unwrap().names.get_random_name();
                                ret_msg = Some("Provided name was invalid, hence random name assigned. Names must not have whitespaces and the '#' char. You may reconnect with a valid name.".to_string());
                            }
                        }
                    }
                    None => {ret_name = shared_state.lock().unwrap().names.get_random_name();ret_msg = None}
                }
                send_packet(&mut stream, ServerPacket::SendGivenName { name: ret_name.clone(), optional_msg: ret_msg });
                {
                    let mut locked_game_data = shared_state.lock().unwrap();
                    let id = locked_game_data.curr_total_clients_num;
                    locked_game_data.curr_total_clients_num += 1;
                    let hand = Hand::new(7, &mut locked_game_data.master_deck); //TODO: Make generic over hand-size
                    locked_game_data.clients_info.push(ClientInfo {
                        id, name: ret_name, hand, stream,
                    })
                }
            }
            _ => server_received_unexpected_packet!(),
        }
        // At this point, the client has connected to the server!
        });
    }
}

pub async fn run_client(port : u32, optional_client_name : Option<&String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(format!("0.0.0.0:{}", port))?;
    dbg!(optional_client_name.clone());
    // Auth loop. Keeps on going if client gives wrong join_code. Ends when it gives right join_code
    let mut is_retry = false; // retry would be when the client sends a wrong code to the server
    loop {
        match read_packet::<ServerPacket>(&mut stream) {
            ServerPacket::AuthRequest { required } => {
                match required {
                    true => {
                        if is_retry {bunt::println!("{$red}Wrong join code{/$}")}
                        bunt::println!("{$yellow}Please provide the join code generated by the server: {/$}");
                        loop {
                            let mut code_str = String::new();
                            io::stdin().read_line(&mut code_str).expect("FATAL ERROR: Could not read line");
                            match code_str.trim().parse::<usize>() {
                                Ok(join_code) => {
                                    send_packet(&mut stream, ClientPacket::AuthResponse { join_code });
                                    is_retry = true;
                                    break;
                                }
                                Err(_) => bunt::println!("{$red}Could not parse input, try again:{/$}")
                            }
                        }
                    }
                    false => {bunt::println!("{$green}Successfully connected to server!{/$}");break;}
                }
            }
            ServerPacket::AuthAcknowledged => {
                bunt::println!("{$green}Successfully connected to server!{/$}");
                break;
            }
            _ => server_received_unexpected_packet!()
        }
    }

    match read_packet::<ServerPacket>(&mut stream) {
        ServerPacket::AskPreferredName => {
            send_packet(&mut stream, ClientPacket::SendPreferredName { optional_client_name: optional_client_name.cloned() })
        }
        _ => client_received_unexpected_packet!()
    }

    match read_packet::<ServerPacket>(&mut stream) {
        ServerPacket::SendGivenName { name, optional_msg } => {
            match optional_msg {
                Some(msg) => {bunt::println!("{$red}{}{/$}", msg)}
                None => (),
            }
            bunt::println!("{$green}Your name is: {}{/$}", name)
        }
        _ => client_received_unexpected_packet!()
    }

    // At this point, the client has connected to the server!
    loop {
        match read_packet::<ServerPacket>(&mut stream) {
            // ServerPacket::AuthRequest { required } => todo!(),
            // ServerPacket::AuthAcknowledged => todo!(),
            // ServerPacket::AskPreferredName => todo!(),
            // ServerPacket::SendGivenName { name, optional_msg } => todo!(),
            ServerPacket::SendMsg { msg } => {println!("{}", msg.unwrap())}
            ServerPacket::SendMsgUpdate { msg, hand, is_my_chance } => {
                println!("{}", msg);
                println!("{}", hand);
                if is_my_chance {
                    println!("It is your turn!\nPlease choose the index of the card you want to play!\n");
                    loop {
                        let mut card_idx_str : String = "".to_string();
                        io::stdin().read_line(&mut card_idx_str).expect("FATAL ERROR: Could not read line");
                        match card_idx_str.trim().parse::<usize>() {
                            Ok(card_idx) if card_idx > 0 && card_idx <= hand.len() => {
                                send_packet(&mut stream, ClientPacket::SendMove { card_idx });
                                break;
                            }
                            _ => bunt::println!("{$red}Invalid Input, try again:{/$}")
                        }
                    }
                }
            }
            _ => {}
        }
    }
    // Ok(())
}
