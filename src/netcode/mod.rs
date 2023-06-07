/* Netcode of the game
 * - For client-server communications, the program uses packets sent over TCP. The packets are
 * essentially PACKET_SIZE sized buffers which are either RequestPacket (client -> server) or
 * ResponsePacket (server -> client) types which are seralized/deserialized on respective ends
 *
 * - For intra-server communications, such serialization/deserialization isn't necessary, hence I
 * simply use Tokio channels for inter-thread communications
 */

pub mod client_server;
pub mod packets;
