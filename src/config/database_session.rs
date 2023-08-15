use futures::future::BoxFuture;
use mongodb::{
    bson::Document,
    error::{
        Result as MongoResult, TRANSIENT_TRANSACTION_ERROR, UNKNOWN_TRANSACTION_COMMIT_RESULT,
    },
    options::{
        FindOneAndUpdateOptions, FindOneOptions, FindOptions, InsertOneOptions, TransactionOptions,
        UpdateOptions,
    },
    ClientSession,
};
use serde::{de::DeserializeOwned, Serialize};

use super::database::{InsertedId, UpdateResult};

#[cfg(test)]
use mockall::mock;

pub struct DbSession(ClientSession);

impl DbSession {
    pub fn new(session: ClientSession) -> Self {
        Self(session)
    }

    pub async fn find_with_session<T>(
        &mut self,
        db: &str,
        coll: &str,
        filter: Option<Document>,
        options: Option<FindOptions>,
    ) -> MongoResult<Vec<T>>
    where
        T: DeserializeOwned + Unpin + Send + Sync,
    {
        let client = self.0.client();
        let collection = client.database(db).collection::<T>(coll);
        let mut cursor = collection
            .find_with_session(filter, options, &mut self.0)
            .await?;
        let mut data = vec![];
        while let Some(doc) = cursor.next(&mut self.0).await {
            data.push(doc?);
        }
        Ok(data)
    }

    pub async fn find_one_with_session<T>(
        &mut self,
        db: &str,
        coll: &str,
        filter: Option<Document>,
        options: Option<FindOneOptions>,
    ) -> MongoResult<Option<T>>
    where
        T: DeserializeOwned + Unpin + Send + Sync,
    {
        let client = self.0.client();
        let collection = client.database(db).collection::<T>(coll);
        collection
            .find_one_with_session(filter, options, &mut self.0)
            .await
    }

    pub async fn find_one_and_update_with_session<T>(
        &mut self,
        db: &str,
        coll: &str,
        filter: Document,
        update: Document,
        options: Option<FindOneAndUpdateOptions>,
    ) -> MongoResult<Option<T>>
    where
        T: DeserializeOwned + Send + Sync,
    {
        let client = self.0.client();
        let collection = client.database(db).collection::<T>(coll);
        collection
            .find_one_and_update_with_session(filter, update, options, &mut self.0)
            .await
    }

    pub async fn insert_one_with_session<T: Serialize>(
        &mut self,
        db: &str,
        coll: &str,
        doc: &T,
        options: Option<InsertOneOptions>,
    ) -> MongoResult<InsertedId> {
        let client = self.0.client();
        let collection = client.database(db).collection::<T>(coll);
        let result = collection
            .insert_one_with_session(doc, options, &mut self.0)
            .await?;
        result.try_into()
    }

    pub async fn update_one_with_session(
        &mut self,
        db: &str,
        coll: &str,
        query: Document,
        update: Document,
        options: Option<UpdateOptions>,
    ) -> MongoResult<UpdateResult> {
        let client = self.0.client();
        let collection = client.database(db).collection::<Document>(coll);
        let result = collection
            .update_one_with_session(query, update, options, &mut self.0)
            .await?;
        result.try_into()
    }

    pub async fn update_many_with_session(
        &mut self,
        db: &str,
        coll: &str,
        query: Document,
        update: Document,
        options: Option<UpdateOptions>,
    ) -> MongoResult<UpdateResult> {
        let client = self.0.client();
        let collection = client.database(db).collection::<Document>(coll);
        let result = collection
            .update_many_with_session(query, update, options, &mut self.0)
            .await?;
        result.try_into()
    }

    pub async fn execute_transaction<F>(
        &mut self,
        options: Option<TransactionOptions>,
        f: F,
    ) -> MongoResult<()>
    where
        F: for<'a> Fn(&'a mut DbSession) -> BoxFuture<'a, MongoResult<()>>,
    {
        self.0.start_transaction(options).await?;
        while let Err(e) = f(self).await {
            if e.contains_label(TRANSIENT_TRANSACTION_ERROR) {
                continue;
            } else {
                self.abort_transaction().await?;
                return Err(e);
            }
        }
        self.commit_transaction().await?;
        Ok(())
    }

    async fn abort_transaction(&mut self) -> MongoResult<()> {
        self.0.abort_transaction().await?;
        Ok(())
    }

    async fn commit_transaction(&mut self) -> MongoResult<()> {
        while let Err(e) = self.0.commit_transaction().await {
            if e.contains_label(UNKNOWN_TRANSACTION_COMMIT_RESULT) {
                continue;
            } else {
                self.abort_transaction().await?;
                return Err(e);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mock! {
    pub DbSession {
        pub fn new(session: ClientSession) -> Self;

        pub async fn execute_transaction<F>(
            &mut self,
            options: Option<TransactionOptions>,
            f: F,
        ) -> MongoResult<()>
        where
            F: for<'a> Fn(&'a mut MockDbSession) -> BoxFuture<'a, MongoResult<()>>,
            F: 'static;

        pub async fn find_with_session<T>(
            &mut self,
            db: &str,
            coll: &str,
            filter: Option<Document>,
            options: Option<FindOptions>,
        ) -> MongoResult<Vec<T>>
        where
            T: DeserializeOwned + Unpin + Send + Sync + 'static;

        pub async fn find_one_with_session<T>(
            &mut self,
            db: &str,
            coll: &str,
            filter: Option<Document>,
            options: Option<FindOneOptions>,
        ) -> MongoResult<Option<T>>
        where
            T: DeserializeOwned + Unpin + Send + Sync + 'static;

        pub async fn find_one_and_update_with_session<T>(
            &mut self,
            db: &str,
            coll: &str,
            filter: Document,
            update: Document,
            options: Option<FindOneAndUpdateOptions>,
        ) -> MongoResult<Option<T>>
        where
            T: DeserializeOwned + Send + Sync + 'static;

        pub async fn insert_one_with_session<T>(
            &mut self,
            db: &str,
            coll: &str,
            doc: &T,
            options: Option<InsertOneOptions>,
        ) -> MongoResult<InsertedId>
        where T: Serialize + 'static;

        pub async fn update_one_with_session(
            &mut self,
            db: &str,
            coll: &str,
            query: Document,
            update: Document,
            options: Option<UpdateOptions>,
        ) -> MongoResult<UpdateResult>;

        pub async fn update_many_with_session(
            &mut self,
            db: &str,
            coll: &str,
            query: Document,
            update: Document,
            options: Option<UpdateOptions>,
        ) -> MongoResult<UpdateResult>;
    }
}
