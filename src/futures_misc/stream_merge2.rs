use futures::{Async, Poll};
use futures::stream::{self, Stream};

pub struct StreamMerge2<S1, S2: Stream> {
    stream1: stream::Fuse<S1>,
    stream2: stream::Fuse<S2>,
    next_try_2: bool,
    queued_error: Option<S2::Error>,
}

pub fn stream_merge2<S1, S2>(stream1: S1, stream2: S2) -> StreamMerge2<S1, S2>
    where S1: Stream, S2: Stream<Error = S1::Error>
{
    StreamMerge2 {
        stream1: stream1.fuse(),
        stream2: stream2.fuse(),
        next_try_2: false,
        queued_error: None,
    }
}

/// An item returned from a merge stream, which represents an item from one or
/// both of the underlying streams.
pub enum Merged2Item<I1, I2> {
    /// An item from the first stream
    First(I1),
    /// An item from the second stream
    Second(I2),
}

fn poll2<S1, S2>(s1: &mut S1, s2: &mut S2)
    -> Poll<Option<Merged2Item<S1::Item, S2::Item>>, S1::Error>
        where
            S1 : Stream,
            S2 : Stream<Error = S1::Error>,
{
    match s1.poll() {
        Err(e) => Err(e),
        Ok(Async::NotReady) => match try_ready!(s2.poll()) {
            Some(item2) => Ok(Async::Ready(Some(Merged2Item::Second(item2)))),
            None => Ok(Async::NotReady),
        },
        Ok(Async::Ready(Some(item1))) => Ok(Async::Ready(Some(Merged2Item::First(item1)))),
        Ok(Async::Ready(None)) => match try_ready!(s2.poll()) {
            Some(item2) => Ok(Async::Ready(Some(Merged2Item::Second(item2)))),
            None => Ok(Async::Ready(None)),
        },
    }
}

impl<S1, S2> Stream for StreamMerge2<S1, S2>
    where S1: Stream, S2: Stream<Error = S1::Error>
{
    type Item = Merged2Item<S1::Item, S2::Item>;
    type Error = S1::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if let Some(e) = self.queued_error.take() {
            return Err(e);
        }

        let r = if self.next_try_2 {
            poll2(&mut self.stream1, &mut self.stream2)
        } else {
            poll2(&mut self.stream2, &mut self.stream1)
                .map(|async: Async<Option<Merged2Item<S2::Item, S1::Item>>>| {
                    async.map(|option| {
                        option.map(|merged_item| {
                            match merged_item {
                                Merged2Item::First(a) => Merged2Item::Second(a),
                                Merged2Item::Second(a) => Merged2Item::First(a),
                            }
                        })
                    })
                })
        };

        self.next_try_2 = !self.next_try_2;

        r
    }
}
