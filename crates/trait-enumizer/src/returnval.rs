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
}


#[cfg(feature = "flume")]
#[cfg_attr(docsrs, doc(cfg(feature = "flume")))]
#[macro_export]
macro_rules! async_flume_class {
    (Sender<$T:ty>) => { ::flume::Sender<$T> };
    (SendError) => { $crate::FailedToSendReturnValue };
    (RecvError) => { ::flume::RecvError };
    (create::<$T:ty>()) => { ::flume::bounded(1) };
    (send::<$T:ty>($channel:expr, $msg:expr)) => { ($channel).send_async($msg).await.map_err(|_| $crate::FailedToSendReturnValue) };
    (recv::<$T:ty>($channel:expr)) => { ($channel).recv_async().await };
}
