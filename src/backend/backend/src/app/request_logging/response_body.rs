use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll, ready},
    time::Instant,
};

use http_body::{Body, Frame};
use pin_project::pin_project;
use tower_http::classify::ClassifyEos;
use tracing::Span;

#[pin_project]

pub struct ResponseBody<B, C> {
    #[pin]
    pub(crate) inner: B,
    pub(crate) classify_eos: Option<C>,
    pub(crate) start: Instant,
    pub(crate) span: Span,
}

impl<B, C> Body for ResponseBody<B, C>
where
    B: Body,
    B::Error: fmt::Display + 'static,
    C: ClassifyEos,
{
    type Data = B::Data;
    type Error = B::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        let _guard = this.span.enter();
        let result = ready!(this.inner.poll_frame(cx));

        //let latency = this.start.elapsed();
        *this.start = Instant::now();

        match result {
            Some(Ok(frame)) => {
                let frame = match frame.into_data() {
                    Ok(chunk) => Frame::data(chunk),
                    Err(frame) => frame,
                };

                let frame = match frame.into_trailers() {
                    Ok(trailers) => Frame::trailers(trailers),
                    Err(frame) => frame,
                };

                Poll::Ready(Some(Ok(frame)))
            }
            Some(Err(err)) => Poll::Ready(Some(Err(err))),
            None => Poll::Ready(None),
        }
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}
