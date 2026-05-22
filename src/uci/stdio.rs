use std::io::{self, BufRead, Write};
use std::sync::mpsc::{Receiver, Sender, channel};

use super::{UciHost, UciRequest, UciResponse};

pub struct StdioUci;

impl UciHost for StdioUci {
    fn start(self) -> (Sender<UciResponse>, Receiver<UciRequest>) {
        let (req_tx, req_rx) = channel::<UciRequest>();
        let (resp_tx, resp_rx) = channel::<UciResponse>();

        std::thread::spawn(move || {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                let Ok(line) = line else { break };
                if let Ok(req) = UciRequest::try_from(line.trim()) {
                    if req_tx.send(req).is_err() {
                        break;
                    }
                }
            }
        });

        std::thread::spawn(move || {
            let stdout = io::stdout();
            for resp in resp_rx {
                let mut out = stdout.lock();
                if writeln!(out, "{}", String::from(&resp)).is_err() {
                    break;
                }
                let _ = out.flush();
            }
        });

        (resp_tx, req_rx)
    }
}
