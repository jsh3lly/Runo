use std::{net::TcpStream, io::{Read, Write}};

use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize, serialize_into};
use bunt::*;

const PACKET_SIZE : usize = 1024;

pub trait Packet{}

#[derive(Serialize, Deserialize)]
pub enum ServerPacket {
    AuthRequest {required: bool},
    AuthAcknowledged,
}

#[derive(Serialize, Deserialize)]
pub enum ClientPacket {
    AuthResponse {join_code : usize},
}

impl Packet for ClientPacket{}
impl Packet for ServerPacket{}


pub fn read_packet<T : for<'a> Deserialize<'a> + Packet>(stream : &mut TcpStream) -> T {
    let mut buff = [0u8; PACKET_SIZE];
    match stream.read_exact(&mut buff) {
        Err(_) => bunt::println!("{$red}Error receiving packet!{/$}"),
        Ok(_) => bunt::println!("{$green}Packet Received!{/$}"),
    }
    deserialize::<T>(&buff).unwrap()
}

pub fn send_packet<T : Serialize + Packet>(stream : &mut TcpStream, packet : T) {
    let mut buff = [0u8; PACKET_SIZE];
    serialize_into(&mut buff[..], &packet).unwrap();
    match stream.write_all(&buff[..PACKET_SIZE]){
        Ok(_) => bunt::println!("{$green}Packet Sent!{/$}"),
        Err(_) => bunt::println!("{$red}Error sending packet{/$}"),
    };
}
