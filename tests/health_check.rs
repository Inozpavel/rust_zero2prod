use maplit::hashmap;
use std::sync::Once;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;
use zero2prod::config::AppConfig;

#[tokio::test]
async fn health_check_works() -> Result<(), anyhow::Error> {
    let base_address = spawn_app().await?;

    let url = format!("{}/health", base_address);
    let response = reqwest::get(url).await?;

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
    Ok(())
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_data() -> Result<(), anyhow::Error> {
    let base_address = spawn_app().await?;
    let url = format!("{}/subscribe", base_address);

    let form = hashmap! {
        "name" => "Le Guin",
        "email" =>"ursula_le_guin@gmail.com"
    };
    let client = reqwest::Client::new();
    let response = client.post(&url).form(&form).send().await?;

    assert!(
        response.status().is_success(),
        "Actual response: {}",
        response.status()
    );
    Ok(())
}
#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() -> Result<(), anyhow::Error> {
    let base_address = spawn_app().await?;
    let url = format!("{}/subscribe", base_address);

    let test_cases = [
        hashmap! {},
        hashmap! { "name" => "Le Guin" },
        hashmap! { "email" => "ursula_le_guin@gmail.com" },
    ];
    let client = reqwest::Client::new();

    for (number, case) in test_cases.into_iter().enumerate() {
        let response = client.post(&url).form(&case).send().await?;

        assert_eq!(
            400,
            response.status().as_u16(),
            "Test case number: {}",
            number
        );
    }
    Ok(())
}

async fn spawn_app() -> Result<String, anyhow::Error> {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let filter = EnvFilter::from(std::env::var("RUST_LOG").unwrap_or("INFO".into()));
        tracing_subscriber::fmt().with_env_filter(filter).init()
    });

    let config = AppConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
    };

    let address = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(address).await?;
    let given_port = listener.local_addr()?.port();

    info!("Listening http://{}", listener.local_addr()?);
    tokio::task::spawn(zero2prod::run(listener));

    let base_address = format!("http://127.0.0.1:{}", given_port);
    Ok(base_address)
}
