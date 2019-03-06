#[macro_use]
extern crate tokio_trace;

use futures::{Async, AsyncSink, Future, Poll, Sink, StartSend, Stream};
use tokio_trace_futures::{Instrument, WithSubscriber};

fn main() {
    let subscriber = tokio_trace_fmt::FmtSubscriber::builder().full().finish();
    tokio_trace_env_logger::try_init().expect("init log adapter");

    tokio_trace::subscriber::with_default(subscriber, || {
        run();
    });
}

fn run() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();

    let stream = futures::stream::iter_ok(0..100);

    let fut = stream.forward(Buf).map(|_| ());

    let span = span!("spanA");
    rt.spawn(fut.instrument(span));

    rt.shutdown_on_idle().wait().unwrap();
}

struct Buf;

impl Sink for Buf {
    type SinkItem = usize;
    type SinkError = ();

    fn start_send(&mut self, _item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        span!("start_send").enter(|| {
            info!("Recieved one item");

            debug!("buffer full, attempting to flush");
            self.poll_complete()?;

            Ok(AsyncSink::Ready)
        })
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        span!("poll_complete").enter(|| {
            debug!("flushing....");
            Ok(Async::NotReady)
        })
    }
}
