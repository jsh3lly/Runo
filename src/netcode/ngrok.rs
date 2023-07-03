use std::io::{BufReader, BufRead};
use std::process::{Command, Child, Stdio, exit};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct NgrokTunnel {
    pub process_handle: Child,
    url: String,
    join_code: String,
}

impl NgrokTunnel {
    pub fn new(port: u32) -> Result<Self, String> {
        let mut ngrok_process  = 
                Command::new("ngrok")
                .arg("tcp")
                .arg("--log=stdout")
                .arg("8080")
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to start ngrok")
                ;

        // Clone the Arc for the termination signal handler
        // let ngrok_process_handler = Arc::clone(&ngrok_process);
        //
        // // Set up termination signal handler
        // ctrlc::set_handler(move || {
        //     // Terminate the ngrok process
        //     let mut ngrok = ngrok_process_handler.lock().unwrap();
        //     ngrok.kill().expect("Failed to kill ngrok");
        //     exit(0);
        // })
        // .expect("Error setting termination signal handler");


        let ngrok_stdout = ngrok_process.stdout.take().unwrap();
        let reader = BufReader::new(ngrok_stdout);
        let mut maybe_url_line = None;
        for line in reader.lines() {
            let line = line.unwrap();
            if line.contains("started tunnel") {
                maybe_url_line = Some(line);
                break;
            }
        }

        ngrok_process.stdout = None;
        let url;
        let join_code;
        match maybe_url_line {
            Some(url_line) => {
                url = url_line.split("url=tcp://").collect::<Vec<&str>>()[1].to_string();
                join_code = url.split(".tcp.ngrok.io:").collect::<Vec<&str>>().join("-");
                let ngrok_tunnel = NgrokTunnel {process_handle: ngrok_process, url, join_code,};
                return Ok(ngrok_tunnel);
            }
            None => {
                return Err(r"Could not start ngrok. Make sure that: \
                    1) You have ngrok installed along with an auth key added to your config file.\
                    2) You are not already running a ngrok tunnel.\ 
                    3) Try a different port".to_string());
            }
        }
        // ngrok_process.lock().unwrap().kill();
    }
    pub fn url(&self) -> String {
        self.url.clone()
    }
    pub fn join_code(&self) -> String {
        self.join_code.clone()
    }

    pub fn close(&mut self) {
        self.process_handle.kill();
    }
}

// impl Drop for NgrokTunnel {
//     fn drop(&mut self) {
//     }
// }

