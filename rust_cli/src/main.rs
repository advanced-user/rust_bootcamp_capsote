use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use std::io;
use std::io::BufRead;
use std::sync::{Arc, Mutex};
use tokio_tungstenite::connect_async;
use tungstenite::protocol::Message;
use uuid::Uuid;

#[derive(Serialize)]
pub enum Msg {
    Left,  // The player moves the character one unit to the left
    Right, // The player moves the character one unit to the right
    Hit,   // A player tries to hit another character
    Wait,  // The player does nothing
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let game_id_str = "8ec45aa2-fa28-42e8-a80c-fd6ea3098424";
    let server_addr = format!("ws://127.0.0.1:8080/{}", game_id_str);

    println!("Connecting to {}", server_addr);


    let (ws_stream, _) = connect_async(server_addr)
        .await
        .expect("Failed to connect to the server");

    let (mut write, mut read) = ws_stream.split();

    println!("Connected to the server!");

    let has_id = Arc::new(Mutex::new(false));
    let user_id = Arc::new(Mutex::new(Uuid::default()));

    let server_task = tokio::spawn({
        let has_id = Arc::clone(&has_id);
        let user_id = Arc::clone(&user_id);
        async move {
            while let Some(Ok(msg)) = read.next().await {
                match msg {
                    Message::Text(text) => {
                        if !*has_id.lock().unwrap() {
                            let message: Vec<&str> = text.split(" ").collect();
                            // "your id is {uuid}"
                            match message.get(3) {
                                None => {}
                                Some(uuid) => {
                                    match Uuid::parse_str(uuid) {
                                        Ok(uuid) => {
                                            *user_id.lock().unwrap() = uuid;
                                            *has_id.lock().unwrap() = true;
                                        }
                                        Err(_) => println!("Invalid Uuid format"),
                                    };
                                    println!("Received message from server: {:?}", uuid.clone());
                                }
                            };
                        } else {
                            println!("Received message from server: {}", text);
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    let send_task = tokio::spawn({
        async move {
            let stdin = io::stdin();
            let mut lines = io::BufReader::new(stdin).lines();
            while let Some(Ok(line)) = lines.next() {
                let msg = match line.trim().to_lowercase().as_str() {
                    "left" => Msg::Left,
                    "right" => Msg::Right,
                    "wait" => Msg::Wait,
                    "hit" => Msg::Hit,
                    _ => continue,
                };

                let json_msg = serde_json::to_string(&msg).expect("");
                let response = write.send(Message::Text(json_msg)).await;

                match response {
                    Err(err) => println!("{}", err),
                    _ => {}
                }
            }
        }
    });

    let _ = tokio::try_join!(send_task, server_task);

    Ok(())
}
