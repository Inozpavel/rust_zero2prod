use maplit::hashmap;

use crate::helpers::spawn_app;

mod helpers;

#[tokio::test]
async fn health_check_works() -> Result<(), anyhow::Error> {
    let app = spawn_app().await?;

    let url = format!("{}/health", app.base_address);
    let response = reqwest::get(url).await?;

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
    Ok(())
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_data() -> Result<(), anyhow::Error> {
    let app = spawn_app().await?;

    let form = hashmap! {
        "name" => "Le Guin",
        "email" =>"ursula_le_guin@gmail.com"
    };
    let response = app.post_subscriptions(&form).await?;

    assert!(
        response.status().is_success(),
        "Actual response: {}",
        response.status()
    );

    let saved = sqlx::query!("SELECT email,name FROM subscriptions")
        .fetch_one(&app.pool)
        .await?;

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "Le Guin");
    Ok(())
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() -> Result<(), anyhow::Error> {
    let app = spawn_app().await?;
    let url = format!("{}/subscriptions", app.base_address);

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
    let test_cases = vec![
        (
            hashmap! { "name" => "", "email" => "ursula_le_guin@gmail.com"},
            "empty name",
        ),
        (hashmap! { "name" => "Ursula", "email" => ""}, "empty email"),
        (
            hashmap! { "name" => "Ursula","email" => "definitely-not-an-email"},
            "empty name",
        ),
    ];

    for (form, description) in test_cases {
        let response = app.post_subscriptions(&form).await?;

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
