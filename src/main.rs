use clap::{command, arg, ArgGroup, value_parser};

// Attaching modules to module tree
mod netcode;
mod card;
mod game;

use crate::netcode::client_server;

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
            arg!(-n --name <VALUE>)
            .help("Enter a name. If not used, a randomly generated name will be used instead!")
            .conflicts_with("server")
            )
        .arg(
            arg!(-s --server)
            .help("Run as server")
            .group("mode")
            )
        .arg(
            arg!(-p --port <VALUE>)
            .help("Specify port number")
            .value_parser(value_parser!(u32).range(1..))
            .conflicts_with("client")
            .default_value("8080")
            )
        .arg(
            arg!(-j --joincode <VALUE>)
            .help("Specify the join code. After the server owner runs the server, say they get the code \"813237\"\n \
                  You need to do `runo -c -j '813237'")
            .required(true)
            .conflicts_with("server")
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
    if *matches.get_one("server").unwrap() {
        client_server::run_server(port).await?;
    }

    if *matches.get_one("client").unwrap() {
        let join_code : String = matches.get_one::<String>("joincode").unwrap().to_string();
        client_server::run_client(matches.get_one("name"), join_code).await?;
    }
    Ok(())
}



