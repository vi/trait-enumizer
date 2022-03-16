#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(docsrs, doc(cfg(feature = "returnval")))]
pub struct FailedToSendReturnValue;
impl std::fmt::Display for FailedToSendReturnValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "trait-enumizer: Failed to send return value back to caller"
        )
    }
}
impl std::error::Error for FailedToSendReturnValue {}

#[cfg(feature = "flume")]
#[cfg_attr(docsrs, doc(cfg(feature = "flume")))]
#[macro_export]
macro_rules! flume_class {
    (Sender<$T:ty>) => { ::flume::Sender<$T> };
    (SendError) => { $crate::FailedToSendReturnValue };
    (RecvError) => { ::flume::RecvError };
    (create::<$T:ty>()) => { ::flume::bounded(1) };
    (send::<$T:ty>($channel:expr, $msg:expr)) => { ($channel).send($msg).map_err(|_| $crate::FailedToSendReturnValue) };
    (recv::<$T:ty>($channel:expr)) => { ($channel).recv() };
    (send_async::<$T:ty>($channel:expr, $msg:expr)) => { ($channel).send_async($msg).await.map_err(|_| $crate::FailedToSendReturnValue) };
    (recv_async::<$T:ty>($channel:expr)) => { ($channel).recv_async().await };
}
#[cfg(feature = "crossbeam-channel")]
#[cfg_attr(docsrs, doc(cfg(feature = "crossbeam-channel")))]
#[macro_export]
macro_rules! crossbeam_class {
    (Sender<$T:ty>) => { ::crossbeam_channel::Sender<$T> };
    (SendError) => { $crate::FailedToSendReturnValue };
    (RecvError) => { ::crossbeam_channel::RecvError };
    (create::<$T:ty>()) => { ::crossbeam_channel::bounded(1) };
    (send::<$T:ty>($channel:expr, $msg:expr)) => { ($channel).send($msg).map_err(|_| $crate::FailedToSendReturnValue) };
    (recv::<$T:ty>($channel:expr)) => { ($channel).recv() };
}


#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
#[macro_export]
macro_rules! tokio_oneshot_class {
    (Sender<$T:ty>) => { ::tokio::sync::oneshot::Sender<$T> };
    (SendError) => { $crate::FailedToSendReturnValue };
    (RecvError) => { ::tokio::sync::oneshot::error::RecvError };
    (create::<$T:ty>()) => { ::tokio::sync::oneshot::channel() };
    (send::<$T:ty>($channel:expr, $msg:expr)) => { ($channel).send($msg).map_err(|_| $crate::FailedToSendReturnValue) };
    (recv::<$T:ty>($channel:expr)) => { ($channel).blocking_recv() };
    (send_async::<$T:ty>($channel:expr, $msg:expr)) => { ($channel).send($msg).map_err(|_| $crate::FailedToSendReturnValue) };
    (recv_async::<$T:ty>($channel:expr)) => { ($channel).await };
}


#[cfg(feature = "catty")]
#[cfg_attr(docsrs, doc(cfg(feature = "catty")))]
#[macro_export]
macro_rules! catty_class {
    (Sender<$T:ty>) => { ::catty::Sender<$T> };
    (SendError) => { $crate::FailedToSendReturnValue };
    (RecvError) => { ::catty::Disconnected };
    (create::<$T:ty>()) => { ::catty::oneshot() };
    (send::<$T:ty>($channel:expr, $msg:expr)) => { ($channel).send($msg).map_err(|_| $crate::FailedToSendReturnValue) };
    (send_async::<$T:ty>($channel:expr, $msg:expr)) => { ($channel).send($msg).map_err(|_| $crate::FailedToSendReturnValue) };
    (recv_async::<$T:ty>($channel:expr)) => { ($channel).await };
}

#[cfg(feature = "futures")]
#[cfg_attr(docsrs, doc(cfg(feature = "futures")))]
#[macro_export]
macro_rules! futures_oneshot_class {
    (Sender<$T:ty>) => { ::futures::channel::oneshot::Sender<$T> };
    (SendError) => { $crate::FailedToSendReturnValue };
    (RecvError) => { ::futures::channel::oneshot::Canceled };
    (create::<$T:ty>()) => { ::futures::channel::oneshot::channel() };
    (send::<$T:ty>($channel:expr, $msg:expr)) => { ($channel).send($msg).map_err(|_| $crate::FailedToSendReturnValue) };
    (send_async::<$T:ty>($channel:expr, $msg:expr)) => { ($channel).send($msg).map_err(|_| $crate::FailedToSendReturnValue) };
    (recv_async::<$T:ty>($channel:expr)) => { ($channel).await };
}

