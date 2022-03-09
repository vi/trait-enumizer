#![cfg(feature="returnval")]
#![feature(generic_associated_types)]
use trait_enumizer::FlumeChannelClass;



#[trait_enumizer::enumizer(returnval=my_rpc_class,call_mut,ref_proxy(unwrapping_impl),enum_attr[derive(serde_derive::Serialize,serde_derive::Deserialize)])]
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
