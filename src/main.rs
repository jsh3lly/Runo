use clap::Parser;

mod card;
mod netstuff;

/// TODO: Uno game
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run the program in server mode
    #[arg(short, long, group="mode"/* , conflicts_with_all=&["client"], required_unless_present="client" */)]
    server: bool,

    /// Run the program in client mode
    #[arg(short, long, group="mode"/* , conflicts_with_all=&["server"], required_unless_present="server" */)]
    client: bool,

    /// Specify port number
    #[arg(short, long, default_value_t=8080)]
    port: u32,

    // /// Specify name (client mode only)
    // #[arg(short, long, requires_if("client", "name not provided"))]
    // name: Option<String>,
}

// TODO: Find better way to do the mutual exclusive mandatory requirement for client and server
impl Args {
    fn validate(&self) -> Result<(), String> {
        if !self.server && !self.client {
            return Err("At least one of `--server` or `--client` must be specified.".to_string());
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = Args::parse();
    args.validate().map_err(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    }).unwrap();

    if args.server {
        netstuff::run_server(args.port).await?;
    }
    else if args.client {
        netstuff::run_client(args.port, /* args.name */).await?;
    }

    Ok(())
}



