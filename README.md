# Runo 
Play Uno on the terminal over the internet with your friends, written in Rust!

[![Crates.io](https://img.shields.io/crates/v/runo-game.svg)](https://crates.io/crates/runo-game)

## Dependencies
* cargo
* ngrok

## Demonstration

https://github.com/StaticESC/Runo/assets/44647704/93b6c890-7fda-4ee7-a1a6-a527818085fe

## Getting started (one-time setup)
### Cargo
To install cargo (and the rust compiler), do `curl https://sh.rustup.rs -sSf | sh`

### Ngrok
To play the game, one of the players will need `ngrok` installed on their machine. This is the only real dependency required for the game to work.
1. Make an account on [ngrok](https://ngrok.com/).
2. Install ngrok on your system.
Say, for arch based systems...
```bash
sudo pacman -S ngrok
```
3. Connect your account by adding your auth-token
```bash
ngrok config add-authtoken <token>
```

### Installing the game
To install the game, do `cargo install runo-game`.

## How to play
### Running the server
This has to be done by only one player and is a very straight-forward procedure. Will take about 5 mins to do.
1. To run the server, first you need to port-forward using ngrok
```bash
ngrok tcp <PORT> # eg, port can be 8080
```
Under `Forwarding:`, take note of the url. For example, it could be: `tcp://0.tcp.ngrok.io:12345`
Then, your join code will be `012345`. Tell this to your friends whom you are playing with. You yourself will need it as well.
2. Then, you need to run the runo server. This can be done by doing...
```bash
runo-game -s -p <PORT> # You do not need to specify port, it chooses 8080 by default. Just make sure the port matches with the one in ngrok.
```
### Running the client
This has to be done by all the players (including the person who runs the server).
```bash
runo-game -c -j <JOIN CODE>
```

Hence, one person has to:
1. Run ngrok. eg: `ngrok tcp 8080` -> `Forwarding: tcp://0.tcp.ngrok.io:12345` -> `Join Code is '012345'`
2. Run the Runo server. `runo-game -s #runs on default port 8080`
3. Run the client `runo-game -c -j "012345` (This has to be done by all players).


Once all clients have connected, the person who's running the server can type `start` on the server terminal to start the game!

## Bug Reporting and Feature Requests
If you encounter any bugs or have ideas for new features, I encourage you to submit them via GitHub issues. Your feedback is valuable and will help me improve the project.

## License
See [LICENSE](https://github.com/StaticESC/Runo/blob/main/LICENSE) for details.

