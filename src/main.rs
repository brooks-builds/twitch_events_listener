use std::{
    sync::mpsc::{self},
    thread,
};

use dotenvy::dotenv;
use twitch_events_listener::{config::Config, run};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config = Config::new_from_env().unwrap();
    let (sender, receiver) = mpsc::channel();

    let receiver_thread = thread::spawn(move || loop {
        let event = receiver.recv().unwrap();

        println!("received a message!!! {event:?}");
    });

    run(config, sender).await.unwrap();

    receiver_thread.join().unwrap();
}
