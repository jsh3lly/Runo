use std::{net::TcpStream, io::{Read, Write}};

use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize, serialize_into};
use bunt::*;

use crate::card::Hand;

const PACKET_SIZE : usize = 1024;

pub trait TCPPacket{}

#[derive(Serialize, Deserialize)]
// Packets sent by the client serving server threads
pub enum ServerPacket {
    AuthRequest {required: bool},
    AuthAcknowledged,
    AskPreferredName,
    SendGivenName {name : String, optional_msg: Option<String>},
    SendMsg {msg: Option<String>},
    SendMsgUpdate {msg: String, hand: Hand, is_my_chance: bool}
}

#[derive(Serialize, Deserialize)]
// Packets sent by the client
pub enum ClientPacket {
    AuthResponse {join_code : usize},
    SendPreferredName {optional_client_name: Option<String>},
    SendMove {card_idx: usize},
}

impl TCPPacket for ClientPacket{}
impl TCPPacket for ServerPacket{}


pub fn read_packet<T : for<'a> Deserialize<'a> + TCPPacket>(stream : &mut TcpStream) -> T {
    let mut buff = [0u8; PACKET_SIZE];
    match stream.read_exact(&mut buff) {
        Err(_) => bunt::println!("{$red}Error receiving packet!{/$}"),
        Ok(_) => bunt::println!("{$green}Packet Received!{/$}"),
    }
    deserialize::<T>(&buff).unwrap()
}

pub fn send_packet<T : Serialize + TCPPacket>(stream : &mut TcpStream, packet : T) {
    let mut buff = [0u8; PACKET_SIZE];
    serialize_into(&mut buff[..], &packet).unwrap();
    match stream.write_all(&buff[..PACKET_SIZE]){
        Ok(_) => bunt::println!("{$green}Packet Sent!{/$}"),
        Err(_) => bunt::println!("{$red}Error sending packet{/$}"),
    };
}

#[derive(Clone, Debug)]
pub enum GameThreadBroadcastPacket {
    StartGame,
}
