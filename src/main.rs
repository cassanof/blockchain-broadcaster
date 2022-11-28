use racketchain_server::{http::HTTP, messages::{NewBlock}};

#[tokio::main]
async fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    let host = args.get(1);
    let port = args.get(2);
    let redis_host = match args.get(3) {
        Some(host) => host.to_string(),
        None => "redis://127.0.0.1/".to_string(),
    };

    let (host, port) = match (host, port) {
        (Some(host), Some(port)) => (host, port),
        _ => {
            eprintln!("Usage: {} HOST PORT [redis-host, defaults to lo]", args[0]);
            std::process::exit(1);
        }
    };

    let client = redis::Client::open(redis_host).expect("Failed to connect to redis");
    let mut con = client.get_connection().expect("Failed to get connection");
    let http = HTTP::new(host.to_string(), port.to_string());
    run_migration_if_needed(&mut con);
    http.start(con).await.expect("Failed to start http server");
}

/// Creates the genesis block if there are no messages in the database.
fn run_migration_if_needed(con: &mut redis::Connection) {
    use redis::Commands;
    let messages: Vec<String> = con.lrange("messages", 0, 1).unwrap();
    if messages.is_empty() {
        let genesis = NewBlock::genesis().to_string();
        let _: () = con.rpush("messages", genesis).unwrap();
    }
}
