#[cfg_attr(docsrs, doc(cfg(feature = "returnval")))]
pub trait SyncChannelClass {
    type Sender<T>;
    type Receiver<T>;
    type SendError;
    type RecvError;
    fn create<T>(&self) -> (Self::Sender<T>, Self::Receiver<T>);
    fn send<T>(&self, s: Self::Sender<T>, msg: T) -> Result<(), Self::SendError>;
    fn recv<T>(&self, r: Self::Receiver<T>) -> Result<T, Self::RecvError>;
}

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
#[cfg_attr(docsrs, doc(cfg(all(feature = "returnval", feature = "flume"))))]
pub struct FlumeChannelClass;

#[macro_export]
macro_rules! flume_class {
    (Sender<$T:ty>) => { ::flume::Sender<$T> };
    (SendError) => { $crate::FailedToSendReturnValue };
    (RecvError) => { ::flume::RecvError };
    (create()) => { ::flume::bounded(1) };
    (send($channel:expr, $msg:expr)) => { ($channel).send($msg).map_err(|_| $crate::FailedToSendReturnValue) };
    (recv($channel:expr)) => { ($channel).recv() };
}

#[cfg(feature = "flume")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "returnval", feature = "flume"))))]
impl SyncChannelClass for FlumeChannelClass {
    type Sender<T> = flume::Sender<T>;
    type Receiver<T> = flume::Receiver<T>;
    type SendError = FailedToSendReturnValue;
    type RecvError = flume::RecvError;

    fn create<T>(&self) -> (Self::Sender<T>, Self::Receiver<T>) {
        flume::bounded(1)
    }

    fn send<T>(&self, s: Self::Sender<T>, msg: T) -> Result<(), Self::SendError> {
        s.send(msg).map_err(|_| FailedToSendReturnValue)
    }

    fn recv<T>(&self, r: Self::Receiver<T>) -> Result<T, Self::RecvError> {
        r.recv()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "returnval")))]
pub struct StdChannelClass;

#[cfg_attr(docsrs, doc(cfg(feature = "returnval")))]
impl SyncChannelClass for StdChannelClass {
    type Sender<T> = std::sync::mpsc::SyncSender<T>;
    type Receiver<T> = std::sync::mpsc::Receiver<T>;
    type SendError = FailedToSendReturnValue;
    type RecvError = std::sync::mpsc::RecvError;

    fn create<T>(&self) -> (Self::Sender<T>, Self::Receiver<T>) {
        std::sync::mpsc::sync_channel(1)
    }

    fn send<T>(&self, s: Self::Sender<T>, msg: T) -> Result<(), Self::SendError> {
        s.send(msg).map_err(|_| FailedToSendReturnValue)
    }

    fn recv<T>(&self, r: Self::Receiver<T>) -> Result<T, Self::RecvError> {
        r.recv()
    }
}

#[cfg(feature = "crossbeam-channel")]
#[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "returnval", feature = "crossbeam-channel")))
)]
pub struct CrossbeamChannelClass;

#[cfg(feature = "crossbeam-channel")]
#[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "returnval", feature = "crossbeam-channel")))
)]
impl SyncChannelClass for CrossbeamChannelClass {
    type Sender<T> = crossbeam_channel::Sender<T>;
    type Receiver<T> = crossbeam_channel::Receiver<T>;
    type SendError = FailedToSendReturnValue;
    type RecvError = crossbeam_channel::RecvError;

    fn create<T>(&self) -> (Self::Sender<T>, Self::Receiver<T>) {
        crossbeam_channel::bounded(1)
    }

    fn send<T>(&self, s: Self::Sender<T>, msg: T) -> Result<(), Self::SendError> {
        s.send(msg).map_err(|_| FailedToSendReturnValue)
    }

    fn recv<T>(&self, r: Self::Receiver<T>) -> Result<T, Self::RecvError> {
        r.recv()
    }
}
