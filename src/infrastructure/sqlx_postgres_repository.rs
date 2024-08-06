use std::fmt::Debug;

use crate::domain::entities::subscriber::Subscriber;
use crate::domain::value_objects::SubscriberId;
use crate::domain::value_objects::{ConfirmationStatus, SubscriberEmail};
use crate::error::{DomainError, RepositoryError};
use chrono::Utc;
use sqlx::PgPool;
use sqlx::Postgres;
use sqlx::Transaction;
use tracing::error;

#[derive(Debug, Clone)]
pub struct SqlxPostgresRepository(PgPool);

impl SqlxPostgresRepository {
    pub fn new(executor: PgPool) -> Self {
        Self(executor)
    }

    pub fn inner(&self) -> &PgPool {
        &self.0
    }
}

impl SqlxPostgresRepository {
    #[tracing::instrument(skip_all)]
    pub async fn begin_transaction(&self) -> Result<Transaction<Postgres>, RepositoryError> {
        let transaction = self.0.begin().await.unwrap();
        Ok(transaction)
    }

    #[tracing::instrument(skip_all)]
    pub async fn get_subscriber_id_by_token(
        &self,
        token: &str,
    ) -> Result<Option<SubscriberId>, RepositoryError> {
        let id = sqlx::query!(
            "SELECT subscriber_id FROM subscription_tokens WHERE subscription_token=$1",
            token
        )
        .fetch_optional(&self.0)
        .await
        .map_err(|e| {
            error!("Failed to execute query: {:?}", e);
            e
        })?
        .map(|x| x.subscriber_id);
        //
        // let id: Option<(String,)> = sqlx::query_as(
        //     "SELECT subscriber_id,a FROM subscription_tokens WHERE subscription_token=$1",
        // )
        // .bind(token)
        // .fetch_optional(&self.0)
        // .await?;

        let id = id
            .map(|id| SubscriberId::parse(&id).map_err(RepositoryError::Domain))
            .transpose()?;

        Ok(id)
    }

    pub async fn get_confirmed_emails(
        &self,
    ) -> Result<Vec<Result<SubscriberEmail, DomainError>>, RepositoryError> {
        let emails = sqlx::query!(
            "SELECT email FROM subscriptions WHERE status=$1",
            ConfirmationStatus::Confirmed.as_ref()
        )
        .fetch_all(&self.0)
        .await?
        .into_iter()
        .map(|x| SubscriberEmail::parse(x.email))
        .collect::<Vec<_>>();

        Ok(emails)
    }

    #[tracing::instrument(skip_all)]
    pub async fn update_subscriber_confirmation_status(
        &self,
        subscriber_id: &SubscriberId,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            "UPDATE subscriptions SET status=$1 WHERE id=$2",
            ConfirmationStatus::Confirmed.as_ref(),
            subscriber_id.as_ref()
        )
        .execute(&self.0)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn insert_subscriber_tx(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        subscriber: &Subscriber,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
        INSERT INTO subscriptions (id, name, email, subscribed_at, status)
        VALUES ($1,$2,$3,$4,$5)
        "#,
            subscriber.id.as_ref(),
            subscriber.name.as_ref(),
            subscriber.email.as_ref(),
            Utc::now(),
            ConfirmationStatus::PendingConfirmation.as_ref()
        )
        .execute(&mut **transaction)
        .await
        .map_err(|e| {
            error!("Failed to execute query {:?}", e);
            e
        })?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn store_token_tx(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        subscriber: &Subscriber,
        token: &str,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
        INSERT INTO subscription_tokens (subscriber_id, subscription_token)
        VALUES ($1, $2)
        "#,
            &subscriber.id.as_ref().to_string(),
            token
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}
