use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

/// 零成本抽象：异步执行器包装器
pub struct Executor<F> {
    inner: F,
}

impl<F> Executor<F> {
    pub const fn new(inner: F) -> Self {
        Self { inner }
    }
}

impl<F: Future> Future for Executor<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { self.map_unchecked_mut(|s| &mut s.inner).poll(cx) }
    }
}

/// 高性能Future组合器
pub trait FutureExt: Future {
    fn map<U, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnOnce(Self::Output) -> U,
    {
        Map { future: self, f }
    }
}

impl<F: Future> FutureExt for F {}

pub struct Map<Fut, F> {
    future: Fut,
    f: F,
}

impl<Fut, F, U> Future for Map<Fut, F>
where
    Fut: Future,
    F: FnOnce(Fut::Output) -> U,
{
    type Output = U;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let future = unsafe { Pin::new_unchecked(&mut this.future) };

        match future.poll(cx) {
            Poll::Ready(output) => {
                let f = unsafe { std::ptr::read(&this.f) };
                Poll::Ready(f(output))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
