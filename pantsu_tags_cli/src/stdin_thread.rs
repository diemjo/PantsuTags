use std::sync::mpsc;
use std::{io, thread};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use crate::AppError;
use crate::common::AppResult;

pub struct StdinThread {
    internal: Option<StdinThreadInternal>,
}

struct StdinThreadInternal {
    stdin_thread_handle: JoinHandle<AppResult<String>>,
    rx: Receiver<AppResult<String>>,
    tx: Sender<AppResult<String>>,
}

impl StdinThread {
    pub fn new() -> StdinThread {
        StdinThread {
            internal: None,
        }
    }

    pub fn read_line(&mut self) -> AppResult<String> {
        if self.internal.is_none() {
            self.launch_thread();
        }
        let thread = &self.internal.unwrap();
        thread.rx.recv()?.map_err(|_| AppError::NoRelevantSauces) // todo: replace error
    }

    pub fn get_tx_rx_ref(&mut self) -> (&Sender<AppResult<String>>, &Receiver<AppResult<String>>) {
        if self.internal.is_none() {
            self.launch_thread();
        }
        let thread = &self.internal.unwrap();
        (&thread.tx, &thread.rx)
    }

    fn launch_thread(&mut self) {
        let (tx, rx) = mpsc::channel();
        let thread = thread::spawn(move || {
            let stdin = io::stdin();
            loop {
                let mut input = String::new();
                let res = match stdin.read_line(&mut input) {
                    Ok(_) => Ok(input),
                    Err(e) => Err(AppError::StdinReadError(e)),
                };
                let _ = tx.send(res);
            }
        });
        self.internal = Some(StdinThreadInternal {
            stdin_thread_handle: thread,
            rx,
            tx,
        });
    }
}