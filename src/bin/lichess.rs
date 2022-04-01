use licoricedev::client::Lichess;

pub fn main() {
    lichess_main();
}

async fn lichess_main() {
    let lichess = Lichess::new(String::from("API_TOKEN"));

    let x = lichess.stream_incoming_events().await;
    // x.unwrap().next()
}
