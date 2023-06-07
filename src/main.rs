use clap::{Parser, ValueEnum, command, arg, ArgGroup, value_parser};

// mod card;
mod netcode;
use crate::netcode::client_server;



// /// TODO: Uno game
// #[derive(Parser, Debug)]
// #[command(author, version, about, long_about = None)]
// struct Args {
//     /// Specify the mode
//     #[arg(value_enum, group="mode")]
//     mode: Mode,
//
//     /// Specify port number
//     #[arg(short, long, default_value_t=8080)]
//     port: u32,
//
//     /// Specify name (client mode only)
//     #[arg(short, long, group="mode", requires_if("mode", "client"))]
//     // #[arg(short, long, requires_if("client", "name not provided"))]
//     name: Option<String>,
// }

// #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
// enum Mode {
//     Client,
//     Server,
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let args = Args::parse();
    let matches = command!()
        .group(ArgGroup::new("mode").required(true).multiple(false))
        .arg(
            arg!(-c --client)
            .help("Run as client")
            .group("mode")
            )
        .arg(
            arg!(-s --server)
            .help("Run as server")
            .group("mode")
            )
            .arg(
                arg!(-o --open)
                .help("Start an Open server")
                .conflicts_with("client")
                )
        .arg(
            arg!(-p --port <VALUE>)
            .help("Specify port number")
            .value_parser(value_parser!(u32).range(1..))
            .default_value("8080"),
            )
        // .arg(
        //     arg!(-v --verbose)
        //     .help("Enable verbose mode")
        //     )
        // .arg(
        //     arg!("name")
        //     .short('n')
        //     .long("name")
        //     // .about("Specify name (client mode only)")
        //     .requires_if("mode", "client"),
        //     )
        .get_matches();

    let port = *matches.get_one("port").unwrap();
    let is_open = *matches.get_one("open").unwrap();
    if *matches.get_one("server").unwrap() {
        client_server::run_server(port, is_open).await?;
    }

    if *matches.get_one("client").unwrap() {
        client_server::run_client(port, /* args.name */).await?;
    }
    // else if args.mode == Mode::Client {
    //     client_server::run_client(args.port, /* args.name */).await?;
    // }

    Ok(())
}



