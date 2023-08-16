use std::time::Duration;

use dotenvy::dotenv;
use futures::{future::BoxFuture, StreamExt};
use mongodb::{
    bson::{Bson, Document},
    error::{Error as MongoError, Result as MongoResult},
    options::{
        AggregateOptions, ClientOptions, DeleteOptions, FindOneAndUpdateOptions, FindOneOptions,
        FindOptions, InsertOneOptions, SessionOptions, TransactionOptions, UpdateOptions,
    },
    results::{InsertOneResult, UpdateResult as MongoUpdateResult},
    Client,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::constants::*;

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use super::database_session::DbSession;

#[cfg(test)]
use mockall::mock;

#[derive(Debug)]
pub struct InsertedId(pub String);

impl TryFrom<InsertOneResult> for InsertedId {
    type Error = MongoError;
    fn try_from(value: InsertOneResult) -> Result<Self, Self::Error> {
        match value.inserted_id {
            Bson::ObjectId(oid) => Ok(InsertedId(oid.to_hex())),
            _ => Err(MongoError::custom("Expected ObjectId here")),
        }
    }
}

#[derive(Debug)]
pub struct UpdateResult {
    pub matched_count: u64,
    pub modified_count: u64,
    pub upserted_id: Option<String>,
}

impl UpdateResult {
    pub fn new(matched_count: u64, modified_count: u64, upserted_id: Option<String>) -> Self {
        Self {
            matched_count,
            modified_count,
            upserted_id,
        }
    }
}

impl TryFrom<MongoUpdateResult> for UpdateResult {
    type Error = MongoError;

    fn try_from(value: MongoUpdateResult) -> Result<Self, Self::Error> {
        let upserted_id = match value.upserted_id {
            None => None,
            Some(uid) => match uid {
                Bson::ObjectId(oid) => Some(oid.to_hex()),
                _ => return Err(MongoError::custom("Expected ObjectId here")),
            },
        };
        Ok(UpdateResult::new(
            value.matched_count,
            value.modified_count,
            upserted_id,
        ))
    }
}

pub struct DbClient(Client);

impl DbClient {
    pub async fn new() -> Self {
        dotenv().ok();
        let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI not found in .env file");
        let min_pool = std::env::var("MONGODB_MIN_POOL_SIZE").unwrap_or_default();
        let max_pool = std::env::var("MONGODB_MAX_POOL_SIZE").unwrap_or_default();
        let min_pool = min_pool.parse::<u32>().unwrap_or(MONGO_MIN_POOL_SIZE);
        let max_pool = max_pool.parse::<u32>().unwrap_or(MONGO_MAX_POOL_SIZE);
        let timeout = Duration::from_secs(MONGO_CONN_TIMEOUT);
        let mut client_options = ClientOptions::parse(uri)
            .await
            .expect("Not able to create mongodb ClientOptions");
        client_options.max_pool_size = Some(max_pool);
        client_options.min_pool_size = Some(min_pool);
        client_options.connect_timeout = Some(timeout);
        let client =
            Client::with_options(client_options).expect("Not able to create mongodb Client");
        Self(client)
    }

    pub async fn find_one<T>(
        &self,
        db: &str,
        coll: &str,
        filter: Option<Document>,
        options: Option<FindOneOptions>,
    ) -> MongoResult<Option<T>>
    where
        T: DeserializeOwned + Unpin + Send + Sync,
    {
        let coll = self.0.database(db).collection::<T>(coll);
        coll.find_one(filter, options).await
    }

    pub async fn find<T>(
        &self,
        db: &str,
        coll: &str,
        filter: Option<Document>,
        options: Option<FindOptions>,
    ) -> MongoResult<Vec<T>>
    where
        T: DeserializeOwned + Unpin + Send + Sync,
    {
        let coll = self.0.database(db).collection::<T>(coll);
        let mut cursor = coll.find(filter, options).await?;
        let mut data = vec![];
        while let Some(doc) = cursor.next().await {
            data.push(doc?);
        }
        Ok(data)
    }

    pub async fn find_one_and_update<T>(
        &self,
        db: &str,
        coll: &str,
        filter: Document,
        update: Document,
        options: Option<FindOneAndUpdateOptions>,
    ) -> MongoResult<Option<T>>
    where
        T: DeserializeOwned + Send + Sync,
    {
        let coll = self.0.database(db).collection::<T>(coll);
        coll.find_one_and_update(filter, update, options).await
    }

    pub async fn insert_one<T>(
        &self,
        db: &str,
        coll: &str,
        doc: &T,
        options: Option<InsertOneOptions>,
    ) -> MongoResult<InsertedId>
    where
        T: Serialize,
    {
        let collection = self.0.database(db).collection::<T>(coll);
        let result = collection.insert_one(doc, options).await?;
        result.try_into()
    }

    pub async fn update_one(
        &self,
        db: &str,
        coll: &str,
        query: Document,
        update: Document,
        options: Option<UpdateOptions>,
    ) -> MongoResult<UpdateResult> {
        let collection = self.0.database(db).collection::<Document>(coll);
        let result = collection.update_one(query, update, options).await?;
        result.try_into()
    }

    pub async fn update_many(
        &self,
        db: &str,
        coll: &str,
        query: Document,
        update: Document,
        options: Option<UpdateOptions>,
    ) -> MongoResult<UpdateResult> {
        let collection = self.0.database(db).collection::<Document>(coll);
        let result = collection.update_many(query, update, options).await?;
        result.try_into()
    }

    pub async fn delete_many(
        &self,
        db: &str,
        coll: &str,
        query: Document,
        options: Option<DeleteOptions>,
    ) -> MongoResult<u64> {
        let collection = self.0.database(db).collection::<Document>(coll);
        let result = collection.delete_many(query, options).await?;
        Ok(result.deleted_count)
    }

    pub async fn aggregate(
        &self,
        db: &str,
        coll: &str,
        pipeline: Vec<Document>,
        options: Option<AggregateOptions>,
    ) -> MongoResult<Vec<Document>> {
        let collection = self.0.database(db).collection::<Document>(coll);
        let mut cursor = collection.aggregate(pipeline, options).await?;
        let mut data = vec![];
        while let Some(doc) = cursor.next().await {
            data.push(doc?);
        }
        Ok(data)
    }

    pub async fn execute_transaction<F>(
        &self,
        session_options: Option<SessionOptions>,
        transaction_options: Option<TransactionOptions>,
        f: F,
    ) -> MongoResult<()>
    where
        F: for<'a> Fn(&'a mut DbSession) -> BoxFuture<'a, MongoResult<()>>,
        F: 'static,
    {
        let mut session = self.start_session(session_options).await?;
        session.execute_transaction(transaction_options, f).await?;
        Ok(())
    }

    async fn start_session(&self, options: Option<SessionOptions>) -> MongoResult<DbSession> {
        let session = self.0.start_session(options).await?;
        let session = DbSession::new(session);
        Ok(session)
    }
}

#[cfg(test)]
mock! {
    pub DbClient {
        pub async fn new() -> Self;

        pub async fn find_one<T>(
            &self,
            db: &str,
            coll: &str,
            filter: Option<Document>,
            options: Option<FindOneOptions>,
        ) -> MongoResult<Option<T>>
        where
            T: DeserializeOwned + Unpin + Send + Sync + 'static;

        pub async fn find<T>(
            &self,
            db: &str,
            coll: &str,
            filter: Option<Document>,
            options: Option<FindOptions>,
        ) -> MongoResult<Vec<T>>
        where
            T: DeserializeOwned + Unpin + Send + Sync + 'static;

        pub async fn find_one_and_update<T>(
            &self,
            db: &str,
            coll: &str,
            filter: Document,
            update: Document,
            options: Option<FindOneAndUpdateOptions>,
        ) -> MongoResult<Option<T>>
        where
            T: DeserializeOwned + Send + Sync + 'static;

        pub async fn insert_one<T>(
            &self,
            db: &str,
            coll: &str,
            doc: &T,
            options: Option<InsertOneOptions>,
        ) -> MongoResult<InsertedId>
        where
            T: Serialize + 'static;

        pub async fn update_one(
            &self,
            db: &str,
            coll: &str,
            query: Document,
            update: Document,
            options: Option<UpdateOptions>,
        ) -> MongoResult<UpdateResult>;

        pub async fn update_many(
            &self,
            db: &str,
            coll: &str,
            query: Document,
            update: Document,
            options: Option<UpdateOptions>,
        ) -> MongoResult<UpdateResult>;

        pub async fn delete_many(
            &self,
            db: &str,
            coll: &str,
            query: Document,
            options: Option<DeleteOptions>,
        ) -> MongoResult<u64>;

        pub async fn aggregate(
            &self,
            db: &str,
            coll: &str,
            pipeline: Vec<Document>,
            options: Option<AggregateOptions>,
        ) -> MongoResult<Vec<Document>>;

        pub async fn execute_transaction<F>(
            &self,
            session_options: Option<SessionOptions>,
            transaction_options: Option<TransactionOptions>,
            f: F,
        ) -> MongoResult<()>
        where
            F: for<'a> Fn(&'a mut DbSession) -> BoxFuture<'a, MongoResult<()>>,
            F: 'static;
    }
}
