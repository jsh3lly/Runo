use std::{net::{TcpListener, TcpStream}, io, sync::{Arc, Mutex}};
use rand::Rng;

use crate::netcode::packets::{send_packet, ServerPacket, read_packet, ClientPacket};
use crate::netcode::misc::Names;

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

enum GamePhase {
    Waiting,
    Starting,
    InGame,
    GameOver,
}

#[derive(Debug)]
struct ClientInfo {
    id: usize,
    name: String,
    // hand: 
}

#[derive(Debug)]
struct GlobalGameData {
    names : Names,
    curr_players_num : usize,
    // master_deck: 
    client_infos: Vec<ClientInfo>
}

pub async fn run_server(port : u32, server_is_open : bool) -> Result<(), Box<dyn std::error::Error>> {
    bunt::println!("{$green}The server has been started{/$}");
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    let mut rng = rand::thread_rng();
    let join_code = rng.gen_range(100_000..999_999);
    if !server_is_open {
        bunt::println!("{$green}Joining code is: {} {/$}", join_code);
    }

    let mut shared_global_game_data = Arc::new(Mutex::new(GlobalGameData {
        names: Names::new(),
        curr_players_num: 0,
        client_infos: vec![],
    }));

    /*
     * Main server loop:
     * Admin can run commands here
     */
    tokio::spawn({
        let shared_state = shared_global_game_data.clone();
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
                    {
                        let shared_state_locked = shared_state.lock().unwrap();
                        dbg!(shared_state_locked);
                    }
                }
            }
        }
    });



    /* Client serving loop 
     * 1 thread per client
     */
    loop {
        let (mut stream, peer_addr) = listener.accept()?;
        let shared_state = shared_global_game_data.clone();

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
                    locked_game_data.curr_players_num += 1;
                    let id = locked_game_data.curr_players_num;
                    locked_game_data.client_infos.push(ClientInfo { id, name: ret_name })
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

    Ok(())
}
