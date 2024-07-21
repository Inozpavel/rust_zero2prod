use crate::helpers::{spawn_app, TestApp};
use maplit::hashmap;
use sqlx::Executor;

mod helpers;

#[tokio::test]
async fn health_check_works() -> Result<(), anyhow::Error> {
    let TestApp { base_address, .. } = spawn_app().await?;

    let url = format!("{}/health", base_address);
    let response = reqwest::get(url).await?;

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
    Ok(())
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_data() -> Result<(), anyhow::Error> {
    let TestApp { base_address, pool } = spawn_app().await?;
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

    let saved = sqlx::query!("SELECT email,name FROM subscriptions")
        .fetch_one(&pool)
        .await?;

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "Le Guin");
    Ok(())
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() -> Result<(), anyhow::Error> {
    let TestApp {
        base_address: address,
        ..
    } = spawn_app().await?;
    let url = format!("{}/subscribe", address);

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

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() -> Result<(), anyhow::Error>
{
    let app = spawn_app().await?;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        let response = client
            .post(&format!("{}/subscribe", &app.base_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}. Body: {}",
            description,
            response.text().await?
        );
    }

    Ok(())
}
