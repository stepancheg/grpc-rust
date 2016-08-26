use futures::*;
use futures::stream::Stream;

pub struct Merge<S1, S2: Stream> {
    stream1: stream::Fuse<S1>,
    stream2: stream::Fuse<S2>,
    next_try_2: bool,
    queued_error: Option<S2::Error>,
}

pub fn new<S1, S2>(stream1: S1, stream2: S2) -> Merge<S1, S2>
    where S1: Stream, S2: Stream<Error = S1::Error>
{
    Merge {
        stream1: stream1.fuse(),
        stream2: stream2.fuse(),
        next_try_2: false,
        queued_error: None,
    }
}

/// An item returned from a merge stream, which represents an item from one or
/// both of the underlying streams.
pub enum MergedItem<I1, I2> {
    /// An item from the first stream
    First(I1),
    /// An item from the second stream
    Second(I2),
}

fn poll2<S1, S2>(s1: &mut S1, s2: &mut S2)
    -> Poll<Option<MergedItem<S1::Item, S2::Item>>, S1::Error>
        where
            S1 : Stream,
            S2 : Stream<Error = S1::Error>,
{
    match s1.poll() {
        Poll::Err(e) => Poll::Err(e),
        Poll::NotReady => match s2.poll() {
            Poll::Err(e) => Poll::Err(e),
            Poll::NotReady => Poll::NotReady,
            Poll::Ok(Some(item2)) => Poll::Ok(Some(MergedItem::Second(item2))),
            Poll::Ok(None) => Poll::NotReady,
        },
        Poll::Ok(Some(item1)) => Poll::Ok(Some(MergedItem::First(item1))),
        Poll::Ok(None) => match s2.poll() {
            Poll::Err(e) => Poll::Err(e),
            Poll::NotReady => Poll::NotReady,
            Poll::Ok(Some(item2)) => Poll::Ok(Some(MergedItem::Second(item2))),
            Poll::Ok(None) => Poll::Ok(None),
        },
    }
}

impl<S1, S2> Stream for Merge<S1, S2>
    where S1: Stream, S2: Stream<Error = S1::Error>
{
    type Item = MergedItem<S1::Item, S2::Item>;
    type Error = S1::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if let Some(e) = self.queued_error.take() {
            return Poll::Err(e);
        }

        let r = if self.next_try_2 {
            poll2(&mut self.stream1, &mut self.stream2)
        } else {
            poll2(&mut self.stream2, &mut self.stream1)
                .map(|option| {
                    option.map(|merged_item| {
                        match merged_item {
                            MergedItem::First(a) => MergedItem::Second(a),
                            MergedItem::Second(a) => MergedItem::First(a),
                        }
                    })
                })
        };

        self.next_try_2 = !self.next_try_2;

        r
    }
}
