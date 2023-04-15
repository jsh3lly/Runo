use clap::Parser;

mod card;
mod netstuff;

use card::*;

/// TODO: Uno game
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run the program in server mode
    #[arg(short, long, group="mode")]
    server: bool,

    /// Run the program in client mode
    #[arg(short, long, group="mode")]
    client: bool,

    /// Specify port number
    #[arg(short, long, default_value_t=8080)]
    port: u32,

    // /// Specify name (client mode only)
    // #[arg(short, long, requires_if("client", "name not provided"))]
    // name: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = Args::parse();

    if args.server {
        netstuff::run_server(args.port).await?;
    }
    else if args.client {
        netstuff::run_client(args.port, /* args.name */).await?;
    }

    Ok(())
}



