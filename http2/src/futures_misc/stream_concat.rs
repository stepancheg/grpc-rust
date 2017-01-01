use futures::stream::Stream;
use futures::Poll;
use futures::Async;
use std::mem;


enum State<S1, S2> {
    First(S1, S2),
    Second(S2),
    Done,
}

pub struct Concat<S1, S2> {
    state: State<S1, S2>
}

pub fn stream_concat<S1, S2>(s1: S1, s2: S2) -> Concat<S1, S2>
    where
        S1 : Stream,
        S2 : Stream<Item=S1::Item, Error=S1::Error>,
{
    Concat { state: State::First(s1, s2) }
}

impl<S1, S2> Stream for Concat<S1, S2>
    where
        S1 : Stream,
        S2 : Stream<Item=S1::Item, Error=S1::Error>,
{
    type Item = S1::Item;
    type Error = S1::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            match self.state {
                State::First(ref mut s1, ref _s2) => match s1.poll() {
                    Err(e) => return Err(e),
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Ok(Async::Ready(Some(r))) => return Ok(Async::Ready(Some(r))),
                    Ok(Async::Ready(None)) => (), // roll
                },
                State::Second(ref mut s2) => match s2.poll() {
                    Err(e) => return Err(e),
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Ok(Async::Ready(Some(r))) => return Ok(Async::Ready(Some(r))),
                    Ok(Async::Ready(None)) => (), // roll
                },
                State::Done => panic!("poll after eof"),
            }

            self.state = match mem::replace(&mut self.state, State::Done) {
                State::First(_s1, s2) => State::Second(s2),
                State::Second(_s2) => State::Done,
                State::Done => unreachable!(),
            };

            if let State::Done = self.state {
                return Ok(Async::Ready(None))
            }
        }
    }
}
