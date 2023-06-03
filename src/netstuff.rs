/* Netcode of the game
 * - For client-server communications, the program uses packets sent over TCP. The packets are
 * essentially PACKET_SIZE sized buffers which are either RequestPacket (client -> server) or
 * ResponsePacket (server -> client) types which are seralized/deserialized on respective ends
 *
 * - For intra-server communications, such serialization/deserialization isn't necessary, hence I
 * simply use Tokio channels for inter-thread communications
 */

use std::net::TcpListener;

use bunt::{self, print, println};

const MAX_PLAYERS_LIMIT : u8 = 10;
const PACKET_SIZE : usize = 1024;

macro_rules! cls {
    () => {
        print!("\x1B[2J\x1b[1;1H");
    }
}
pub async fn run_server(port : u32) -> Result<(), Box<dyn std::error::Error>> {
    cls!();
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;

    /*
     * Main server loop:
     * - Logs events from self (server) and clients
     * - Admin user can also run commands
     */

    loop {
        let (mut stream, peer_addr) = listener.accept()?;

        // for every new connection
        tokio::spawn(async move {
        });
    }
}

pub async fn run_client(port : u32) -> Result<(), Box<dyn std::error::Error>> {
    cls!();

    Ok(())
}
