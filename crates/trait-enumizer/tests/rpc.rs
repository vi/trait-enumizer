use std::sync::{Arc, Mutex};

#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
struct ReturnValue<T> {
    value: T,
    ret: usize,
}

#[derive(Clone)]
struct MyRpcClient {
    pending_replies: Arc<Mutex<slab::Slab<flume::Sender<serde_json::Value>>>>,
}

impl MyRpcClient {
    fn allocate(&self) -> ( usize, flume::Receiver<serde_json::Value> ) {
        let (tx,rx) = flume::bounded(1);
        let id =self.pending_replies.lock().unwrap().insert(tx);
        (id, rx)
    }

    fn handle_reply(&self, val: String) {
        let msg : ReturnValue<serde_json::Value> = serde_json::from_str(&val).unwrap();
        let tx = self.pending_replies.lock().unwrap().remove(msg.ret);
        tx.send(msg.value).unwrap();
    }
}

macro_rules! my_rpc_class {
    (Sender<$T:ty>) => { usize };
    (SendError) => { trait_enumizer::FailedToSendReturnValue };
    (RecvError) => { ::flume::RecvError };
    (create::<$T:ty>($c:expr)) => { $c.allocate() };
    (send::<$T:ty>($id:expr, $msg:expr, $tx:expr)) => { {
        let x = ReturnValue{value: $msg, ret: $id};
        let s = serde_json::to_string(&x).unwrap();
        ($tx).send(s).map_err(|_| trait_enumizer::FailedToSendReturnValue)
    } };
    (recv::<$T:ty>($rx:expr, $c:expr)) => { {
        match ($rx).recv() {
            Ok(x) => Ok(serde_json::from_value(x).unwrap()),
            Err(e) => Err(e),
        }
    } };
}

#[trait_enumizer::enumizer(
    name=MyIfaceEnum,
    returnval=my_rpc_class,
    call_fn(name=try_call_mut,ref_mut,extra_arg_type(&flume::Sender<String>)),
    proxy(Fn,name=MyIfaceProxy,unwrapping_impl,extra_field_type(MyRpcClient)),
    enum_attr[derive(serde_derive::Serialize,serde_derive::Deserialize)]
)]
pub trait MyIface {
    fn addone(&mut self);
    fn double(&mut self);
    fn divide(&mut self, denominator: i32);
    fn get(&self) -> i32;
    fn format(&self, pre: String, post: String) -> String;
    fn sleep_without_caller_waiting_for_it(&self, ms:usize);
    fn sleep_making_caller_wait_for_it(&self, ms:usize) -> ();
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

    fn divide(&mut self, denominator: i32) {
        self.0 /= denominator;
    }

    fn format(&self, pre: String, post: String) -> String {
        format!("{}{}{}", pre, self.0, post)
    }

    fn sleep_without_caller_waiting_for_it(&self, ms:usize) {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64))
    }

    fn sleep_making_caller_wait_for_it(&self, ms:usize) -> () {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64))
    }
}



#[test]
fn test() {
    let (tx1, rx1) = flume::bounded::<String>(1);
    let (tx2, rx2) = flume::bounded::<String>(1);

    std::thread::spawn(move || {
        let mut o = Implementor(100);
        for msg in rx1 {
            eprintln!("> {}", msg);
            let x : MyIfaceEnum = serde_json::from_str(&msg).unwrap();
            x.try_call_mut(&mut o, &tx2).unwrap();
        }
    });
    
    let c = MyRpcClient {pending_replies:Arc::new(Mutex::new(slab::Slab::new()))};
    let c2 = c.clone();

    std::thread::spawn(move || {
        for x in rx2 {
            eprintln!("< {}", x);
            c2.handle_reply(x);
        }
    });

    let tx1_ = tx1.clone();
    let c_ = c.clone();
    std::thread::spawn(move || {
        let p = MyIfaceProxy::<_, _>(|msg| tx1_.send(serde_json::to_string(&msg).unwrap()), c_);
        p.sleep_making_caller_wait_for_it(5);
        dbg!(p.get());
        p.sleep_making_caller_wait_for_it(5);
        dbg!(p.get());
        p.sleep_making_caller_wait_for_it(5);
        dbg!(p.get());
        p.sleep_making_caller_wait_for_it(5);
        dbg!(p.get());
    });

    let mut p = MyIfaceProxy::<_, _>(|msg| tx1.send(serde_json::to_string(&msg).unwrap()), c);
    p.addone();
    p.addone();
    dbg!(p.get());
    p.double();
    p.sleep_without_caller_waiting_for_it(20);
    dbg!(p.get());
    p.divide(3);
    eprintln!("{}", p.format("[[[".to_owned(), "]]]".to_owned()));
}
