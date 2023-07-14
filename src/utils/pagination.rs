use std::marker::PhantomData;

use async_trait::async_trait;
use sqlx::{Executor, Postgres};

#[async_trait]
pub trait Paginable<T>: Sized {
    fn paginate(per_page: i64) -> Pages<T, Self> {
        Pages {
            per_page,
            phantom_pages: PhantomData,
            phantom_paginable: PhantomData,
        }
    }

    async fn get_page(
        pages: &Pages<T, Self>,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<T>, sqlx::Error>;
}

pub struct Pages<T, P: Paginable<T>> {
    pub per_page: i64,
    phantom_pages: PhantomData<Vec<Page<T>>>,
    phantom_paginable: PhantomData<P>,
}

impl<T, P: Paginable<T>> Pages<T, P> {
    pub async fn get_page(
        &self,
        page_no: i64,
        connection: impl Executor<'_, Database = Postgres>,
    ) -> Result<Page<T>, sqlx::Error> {
        Paginable::<T>::get_page(self, page_no, connection).await
    }
}

pub struct Page<T> {
    pub per_page: i64,
    pub page_no: i64,
    pub items: Vec<T>
}
