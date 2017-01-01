use futures::stream::Stream;
use futures::Poll;

use super::stream_concat::Concat;
use super::stream_concat::stream_concat;


pub struct Concat3<S1, S2, S3>(Concat<S1, Concat<S2, S3>>);

pub fn stream_concat3<S1, S2, S3>(s1: S1, s2: S2, s3: S3) -> Concat3<S1, S2, S3>
    where
        S1 : Stream,
        S2 : Stream<Item=S1::Item, Error=S1::Error>,
        S3 : Stream<Item=S1::Item, Error=S1::Error>,
{
    Concat3(stream_concat(s1, stream_concat(s2, s3)))
}

impl<S1, S2, S3> Stream for Concat3<S1, S2, S3>
    where
        S1 : Stream,
        S2 : Stream<Item=S1::Item, Error=S1::Error>,
        S3 : Stream<Item=S1::Item, Error=S1::Error>,
{
    type Item = S1::Item;
    type Error = S1::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.0.poll()
    }
}
