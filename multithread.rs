use std::sync::mpsc;
use std::thread;

/*

fn main() {
    let channels = init_channels(8);

    loop {
        for channel in channels.iter() {
            while let Ok(msg) = channel.try_recv() {
                match msg {
                    // handle messages
                }
            }
        }
    }
}

*/

pub enum MessageFromMain {
    //
}

pub enum MessageToMain {
    //
}

pub struct Channel {
    tx_from_main: mpsc::Sender<MessageFromMain>,
    rx_to_main: mpsc::Receiver<MessageToMain>,
}

impl Channel {
    pub fn send(&self, msg: MessageFromMain) -> Result<(), mpsc::SendError<MessageFromMain>> {
        self.tx_from_main.send(msg)
    }

    pub fn try_recv(&self) -> Result<MessageToMain, mpsc::TryRecvError> {
        self.rx_to_main.try_recv()
    }

    pub fn block_recv(&self) -> Result<MessageToMain, mpsc::RecvError> {
        self.rx_to_main.recv()
    }
}

pub fn init_channels(n: usize) -> Vec<Channel> {
    (0..n).map(|_| init_channel()).collect()
}

pub fn init_channel() -> Channel {
    let (tx_to_main, rx_to_main) = mpsc::channel();
    let (tx_from_main, rx_from_main) = mpsc::channel();

    thread::spawn(move || {
        event_loop(tx_to_main, rx_from_main);
    });

    Channel {
        rx_to_main, tx_from_main
    }
}

pub fn distribute_messages(
    messages: Vec<MessageFromMain>,
    channels: &[Channel],
) -> Result<(), mpsc::SendError<MessageFromMain>> {
    for (index, message) in messages.into_iter().enumerate() {
        channels[index % channels.len()].send(message)?;
    }

    Ok(())
}

pub fn event_loop(tx_to_main: mpsc::Sender<MessageToMain>, rx_from_main: mpsc::Receiver<MessageFromMain>) {
    for msg in rx_from_main {
        match msg {
            //
        }
    }

    drop(tx_to_main)
}

