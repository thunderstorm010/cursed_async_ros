pub use rosrust;

use async_stream::stream;
use rosrust::Subscriber;
use tokio::{sync::mpsc, task::spawn_blocking};
pub use tokio_stream;
use tokio_stream::Stream;

pub async fn subscribe<T: rosrust::Message>(
    topic: &'static str,
    queue_size: usize,
) -> (impl Stream<Item = T>, rosrust::error::Result<Subscriber>) {
    let (tx, mut rx) = mpsc::channel::<T>(100);

    let result = spawn_blocking(move || {
        rosrust::subscribe(topic, queue_size, move |message: T| {
            tx.blocking_send(message).unwrap();
        })
    })
    .await
    .unwrap();

    let stream = stream! {
        for msg in rx.recv().await {
            yield msg;
        }
    };

    (stream, result)
}
