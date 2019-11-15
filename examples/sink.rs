use vessels::{
    channel::IdChannel,
    core,
    core::{executor::Spawn, Executor},
    format::{ApplyDecode, ApplyEncode, Json},
    kind::Sink,
    log, OnTo,
};

use futures::{stream::iter, future::pending, StreamExt,SinkExt, channel::mpsc::{channel}};

fn main() {
    let (sender, mut receiver) = channel(0);
    let sender: Sink<u32, ()> = Box::pin(sender.sink_map_err(|_| panic!()));
    core::<dyn Executor>().unwrap().run(async move {
        core::<dyn Executor>().unwrap().spawn(async move {
            while let Some(item) = receiver.next().await {
                log!("{}", item);
            }
        });
        let encoded = sender.on_to::<IdChannel>().await.encode::<Json>();
        let mut decoded: Sink<u32, ()> = encoded.decode::<IdChannel, Json>().await.unwrap();
        decoded.send_all(&mut iter(1..10)).await.unwrap();
        pending::<()>().await;
    });
}