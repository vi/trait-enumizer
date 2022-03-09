#![cfg(feature="returnval")]
#![feature(generic_associated_types)]
use trait_enumizer::FlumeChannelClass;

#[trait_enumizer::enumizer(returnval,call_mut,ref_proxy(unwrapping_impl),enum_attr[derive(serde_derive::Serialize,serde_derive::Deserialize)])]
pub trait MyIface {
    fn addone(&mut self);
    fn double(&mut self);
    fn get(&self) -> i32;
}

pub struct Implementor(pub i32);

impl MyIface for Implementor {
    fn addone(&mut self) {
        self.0 += 1;
    }

    fn double(&mut self) {
        self.0 *= 2;
    }

    fn get(&self) -> i32 {
        self.0
    }
}

struct MyRpcServerClass {
    tx: flume::Sender<serde_json::Value>,
}

impl MyRpcServerClass {
    fn new(client_channel: flume::Sender<serde_json::Value>) -> MyRpcServerClass {
        MyRpcServerClass {
            tx: client_channel,
        }
    }
}

impl trait_enumizer::SyncChannelClass for MyRpcServerClass {
    type Sender<T> = usize;
    type Receiver<T> = std::convert::Infallible;
    type SendError = std::convert::Infallible;
    type RecvError = std::convert::Infallible;

    fn create<T>(&self) -> (Self::Sender<T>, Self::Receiver<T>) {
        unreachable!()
    }

    fn send<T>(&self, s: Self::Sender<T>, msg: T) -> Result<(), Self::SendError> {
        Ok(self.tx.send(serde_json::to_value(msg).unwrap()).unwrap())
    }

    fn recv<T>(&self, r: Self::Receiver<T>) -> Result<T, Self::RecvError> {
        unreachable!()
    }
}

#[test]
fn test() {
    let (tx1, rx1) = flume::bounded::<serde_json::Value>(1);
    let (tx2, rx2) = flume::bounded::<serde_json::Value>(1);

    std::thread::spawn(move || {
        let mut o = Implementor(100);
        let cc = MyRpcServerClass::new(tx2);
        for msg in rx1 {
            eprintln!("> {}", serde_json::ser::to_string(&msg).unwrap());
            let msg : MyIfaceEnum<MyRpcServerClass> = serde_json::from_value(msg).unwrap();
            msg.try_call_mut(&mut o, &cc).unwrap();
        }
    });
    let mut p = MyIfaceProxy::<_, _, _>(|c| tx1.send(c), FlumeChannelClass);
    p.addone();
    p.addone();
    dbg!(p.get());
    p.double();
    dbg!(p.get());
}
