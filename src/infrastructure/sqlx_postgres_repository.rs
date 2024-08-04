use std::fmt::Debug;
use std::future::Future;

use crate::domain::entities::subscriber::Subscriber;
use crate::domain::value_objects::ConfirmationStatus;
use crate::domain::value_objects::SubscriberId;
use crate::error::RepositoryError;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::{Executor, PgPool, Postgres, Transaction};
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
    pub async fn get_subscriber_id_by_token(
        &self,
        token: &str,
    ) -> Result<Option<SubscriberId>, RepositoryError> {
        let id = sqlx::query!(
            "SELECT subscriber_id FROM subscription_tokens WHERE subscription_token=$1",
            token
        )
        .fetch_optional(&self.0)
        .await?
        .map(|x| x.subscriber_id);

        let id = id
            .map(|id| SubscriberId::parse(&id).map_err(RepositoryError::DomainError))
            .transpose()?;

        Ok(id)
    }

    #[tracing::instrument(skip_all)]
    pub async fn begin_transaction(&self) -> Result<Transaction<Postgres>, sqlx::Error> {
        let transaction = self.0.begin().await.unwrap();
        Ok(transaction)
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
    ) -> Result<(), sqlx::Error> {
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
    ) -> Result<(), sqlx::Error> {
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
