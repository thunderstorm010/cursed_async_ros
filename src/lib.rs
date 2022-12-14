use std::sync::Arc;

pub use rosrust;

use async_stream::stream;
use rosrust::{Subscriber, Service, ServicePair};
use tokio::{sync::mpsc, task::spawn_blocking};
pub use tokio_stream;
use tokio_stream::Stream;




pub async fn subscribe<T: rosrust::Message>(
    topic: &'static str,
    queue_size: usize,
) -> rosrust::error::Result<impl Stream<Item = T>> {
    let (tx, mut rx) = mpsc::channel::<T>(100);

    spawn_blocking(move || {
        rosrust::subscribe(topic, queue_size, move |message: T| {
            tx.blocking_send(message).unwrap();
        }).unwrap();
        rosrust::spin();
    })
    .await.unwrap();

    let stream = stream! {
        while let Some(msg) = rx.recv().await {
            yield msg;
        }
    };

    Ok(stream)
}

pub async fn service<T: rosrust::ServicePair, F: Fn(T::Request) -> Result<T::Response, String> + Send + Sync + 'static> (
    service: &'static str,
    callback: F
) -> Result<Service, rosrust::error::Error> {
    spawn_blocking(move || {
        rosrust::service::<T, F>(service, callback)
    }).await.unwrap()
}

#[derive(Clone)]
pub struct Client<T: ServicePair + Clone> {
    inner: Arc<rosrust::Client<T>>
}

impl<T: ServicePair + Clone> Client<T> {
    pub async fn req(&self, args: T::Request) -> rosrust::error::tcpros::Result<Result<T::Response, String>> {
       let inner = self.inner.clone();
        spawn_blocking(move || {
            inner.req(&args)
        }).await.unwrap()
    }
}

pub async fn client<T: ServicePair + Clone>(service: &'static str) -> Result<Client<T>, rosrust::error::Error> {
    spawn_blocking(move || {
        rosrust::client::<T>(service)
    }).await.unwrap().map(|cl| Client { inner: Arc::new(cl) })
}