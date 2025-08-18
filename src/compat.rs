// thank you to async compat!
// this is a stripped version of the crate with the alive thread handled by bevy
use std::pin::Pin;
use std::task::{Context, Poll};

pub fn alive() {
    #[cfg(not(target_family = "wasm"))]
    std::thread::spawn(|| TOKIO.block_on(Pending));
    #[cfg(target_family = "wasm")]
    bevy::tasks::IoTaskPool::get()
        .spawn(async { TOKIO.block_on(Pending) })
        .detach();
}

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct Compat<T> {
        #[pin]
        inner: Option<T>,
    }

    impl<T> PinnedDrop for Compat<T> {
        fn drop(this: Pin<&mut Self>) {
            if this.inner.is_some() {
                let _guard = get_runtime_handle().enter();
                this.project().inner.set(None);
            }
        }
    }
}

impl<T> Compat<T> {
    pub fn new(t: T) -> Compat<T> {
        Compat { inner: Some(t) }
    }
    fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut T> {
        self.project().inner.as_pin_mut().unwrap()
    }
}

impl<T: Future> Future for Compat<T> {
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let _guard = get_runtime_handle().enter();
        self.get_pin_mut().poll(cx)
    }
}

fn get_runtime_handle() -> tokio::runtime::Handle {
    tokio::runtime::Handle::try_current().unwrap_or_else(|_| TOKIO.handle().clone())
}

static TOKIO: std::sync::LazyLock<tokio::runtime::Runtime> = std::sync::LazyLock::new(|| {
    #[cfg(not(target_family = "wasm"))]
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    #[cfg(target_family = "wasm")]
    let mut builder = tokio::runtime::Builder::new_current_thread();
    builder.enable_all().build().unwrap()
});

struct Pending;

impl Future for Pending {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}
