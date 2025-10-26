use chrono::Duration;
use std::future::Future;
use std::pin::Pin;
use tokio_util::sync::CancellationToken;

use crate::state::AppState;

#[macro_export]
macro_rules! make_interval_handler {
    ($async_fn:expr) => {{
        fn wrapper(
            state: &$crate::state::AppState,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
            Box::pin($async_fn(state))
        }
        wrapper
    }};
}

pub fn interval_handler<H>(
    state: AppState,
    duration_secs: Duration,
    ct: CancellationToken,
    handler: H,
) where
    H: for<'a> Fn(&'a AppState) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send + 'static,
{
    tokio::spawn(async move {
        let dur = duration_secs.to_std().unwrap();
        loop {
            tokio::select! {
                _ = handler(&state) => {
                    tokio::time::sleep(dur.clone()).await;
                }
                _ = ct.cancelled() => {
                    break;
                }
            }
        }
    });
}
