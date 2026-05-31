use anyhow::Result;
use clap::Parser;
use futures::{SinkExt, StreamExt};
use std::io::{self, Write};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

#[derive(Parser, Debug)]
#[command(author, version, about = "Interactive CLI that can send messages over WebSocket")]
struct Args {
    /// Server WebSocket URL
    #[arg(short, long, default_value = "ws://127.0.0.1:8000/ws")]
    url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let url = Url::parse(&args.url)?;

    println!("Connecting to {}...", url);
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // spawn a task to print incoming messages from server
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(t)) => println!("< server: {}", t),
                Ok(Message::Close(_)) => {
                    println!("server closed connection");
                    break;
                }
                _ => {}
            }
        }
    });

    println!("--- Interactive CLI ---");
    println!("Type coordinates like: 124, 154  (or send <raw> for raw JSON)");
    println!("Type 'exit' to quit.");

    loop {
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let line = input.trim();
        if line.is_empty() {
            continue;
        }

        if line == "exit" || line == "quit" {
            break;
        }

        if let Some(rest) = line.strip_prefix("send ") {
            let txt = rest.trim();
            if !txt.is_empty() {
                let _ = write.send(Message::Text(txt.to_string())).await;
            }
            continue;
        }

        // Try to parse coordinates: "x, y" or "x,y"
        if let Some((x_str, y_str)) = line.split_once(',') {
            let x: f64 = match x_str.trim().parse() {
                Ok(v) => v,
                Err(_) => {
                    println!("invalid x coordinate: '{}'", x_str.trim());
                    continue;
                }
            };
            let y: f64 = match y_str.trim().parse() {
                Ok(v) => v,
                Err(_) => {
                    println!("invalid y coordinate: '{}'", y_str.trim());
                    continue;
                }
            };
            let cmd = format!(r#"{{"x_pos": {}, "y_pos": {}}}"#, x, y);
            println!(">> {}", cmd);
            let _ = write.send(Message::Text(cmd.into())).await;
            continue;
        }

        println!("unknown input. Use: x, y  (e.g. 124, 154)");
    }

    Ok(())
}