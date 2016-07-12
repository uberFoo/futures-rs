use std::sync::Arc;

use {Wake, Tokens, ALL_TOKENS};
use stream::{Stream, StreamResult};

pub struct SkipWhile<S, P> {
    stream: S,
    pred: P,
    done_skipping: bool,
}

pub fn new<S, P>(s: S, p: P) -> SkipWhile<S, P>
    where S: Stream,
          P: FnMut(&S::Item) -> Result<bool, S::Error> + Send + 'static
{
    SkipWhile {
        stream: s,
        pred: p,
        done_skipping: false,
    }
}

impl<S, P> Stream for SkipWhile<S, P>
    where S: Stream,
          P: FnMut(&S::Item) -> Result<bool, S::Error> + Send + 'static
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self, mut tokens: &Tokens)
            -> Option<StreamResult<S::Item, S::Error>> {
        if self.done_skipping {
            return self.stream.poll(tokens);
        }

        loop {
            let item = match self.stream.poll(tokens) {
                Some(Ok(Some(e))) => e,
                Some(Ok(None)) => return Some(Ok(None)),
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            };
            match (self.pred)(&item) {
                Ok(false) => {
                    self.done_skipping = true;
                    return Some(Ok(Some(item)))
                }
                Ok(true) => {}
                Err(e) => return Some(Err(e)),
            }
            tokens = &ALL_TOKENS;
        }
    }

    fn schedule(&mut self, wake: Arc<Wake>) {
        self.stream.schedule(wake)
    }
}

impl<S, P> SkipWhile<S, P> {
    // TODO: why here and not elsewhere...
    pub fn into_inner(self) -> S {
        self.stream
    }
}
