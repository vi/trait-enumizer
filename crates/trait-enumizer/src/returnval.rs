pub trait SyncChannelClass {
    type Sender<T>;
    type Receiver<T>;
    type SendError;
    type RecvError;
    fn create<T>(&self) -> (Self::Sender<T>, Self::Receiver<T>);
    fn send<T>(s: Self::Sender<T>, msg: T) -> Result<(), Self::SendError>;
    fn recv<T>(r: Self::Receiver<T>) -> Result<T, Self::RecvError>;
}

#[derive(Debug,Clone, Copy,PartialEq, Eq, PartialOrd, Ord,Hash)]
pub struct FailedToSendReturnValue;
impl std::fmt::Display for FailedToSendReturnValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "trait-enumizer: Failed to send return value back to caller")
    }
}
impl std::error::Error for FailedToSendReturnValue {}

#[cfg(feature="flume")]
pub struct FlumeChannelClass;

#[cfg(feature="flume")]
impl SyncChannelClass for FlumeChannelClass {
    type Sender<T> = flume::Sender<T>;
    type Receiver<T> = flume::Receiver<T>;
    type SendError = FailedToSendReturnValue;
    type RecvError = flume::RecvError;

    fn create<T>(&self) -> (Self::Sender<T>, Self::Receiver<T>) {
        flume::bounded(1)
    }

    fn send<T>(s: Self::Sender<T>, msg: T) -> Result<(), Self::SendError> {
        s.send(msg)
            .map_err(|_| FailedToSendReturnValue)
    }

    fn recv<T>(r: Self::Receiver<T>) -> Result<T, Self::RecvError> {
        r.recv()
    }
}
