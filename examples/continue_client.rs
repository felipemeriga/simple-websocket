use rand::distr::Alphanumeric;
use rand::{thread_rng, Rng};
use socket_flow::handshake::connect_async;

async fn handle_connection(addr: String) {
    match connect_async(addr).await {
        Ok(mut ws_connection) => {
            let my_random_string = generate_random_string();
            println!("Sending random string: {}", my_random_string);
            if ws_connection
                .send_large_data_fragmented(Vec::from(my_random_string))
                .await
                .is_err()
            {
                eprintln!("Error occurred when sending data in chunks");
            }

            ws_connection.close_connection().await.unwrap();
        }
        Err(err) => eprintln!("Error when performing handshake: {}", err),
    }
}

#[tokio::main]
async fn main() {
    handle_connection(String::from("127.0.0.1:9002")).await;
}

fn generate_random_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}
