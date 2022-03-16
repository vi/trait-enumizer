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
    fn allocate(&self) -> (usize, flume::Receiver<serde_json::Value>) {
        let (tx, rx) = flume::bounded(1);
        let id = self.pending_replies.lock().unwrap().insert(tx);
        (id, rx)
    }

    async fn handle_reply(&self, val: String) {
        let msg: ReturnValue<serde_json::Value> = serde_json::from_str(&val).unwrap();
        let tx = {self.pending_replies.lock().unwrap().remove(msg.ret)};
        tx.send_async(msg.value).await.unwrap();
    }
}

macro_rules! my_async_rpc_class {
    (Sender<$T:ty>) => {
        usize
    };
    (SendError) => {
        trait_enumizer::FailedToSendReturnValue
    };
    (RecvError) => {
        ::flume::RecvError
    };
    (create::<$T:ty>($c:expr)) => {
        $c.allocate()
    };
    (send_async::<$T:ty>($id:expr, $msg:expr, $tx:expr)) => {{
        let x = ReturnValue {
            value: $msg,
            ret: $id,
        };
        let s = serde_json::to_string(&x).unwrap();
        ($tx)
            .send_async(s)
            .await
            .map_err(|_| trait_enumizer::FailedToSendReturnValue)
    }};
    (recv_async::<$T:ty>($rx:expr, $c:expr)) => {{
        match ($rx).recv_async().await {
            Ok(x) => Ok(serde_json::from_value(x).unwrap()),
            Err(e) => Err(e),
        }
    }};
}

pub struct Implementor(pub i32);

#[trait_enumizer::enumizer(
    name=MyIfaceEnum,
    inherent_impl,
    returnval=my_async_rpc_class,
    call_fn(name=try_call_mut,async,ref_mut,extra_arg_type(&flume::Sender<String>)),
    proxy(Fn,name=MyIfaceProxy,extra_field_type(MyRpcClient),async),
    enum_attr[derive(serde_derive::Serialize,serde_derive::Deserialize)],
)]
impl Implementor {
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

    async fn format(&self, pre: String, post: String) -> String {
        format!("{}{}{}", pre, self.0, post)
    }

    async fn sleep_without_caller_waiting_for_it(&self, ms: usize) {
        tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await
    }

    async fn sleep_making_caller_wait_for_it(&self, ms: usize) -> () {
        tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await
    }
}

#[tokio::test]
async fn test() {
    let (tx1, rx1) = flume::bounded::<String>(1);
    let (tx2, rx2) = flume::bounded::<String>(1);

    tokio::spawn(async move {
        let mut o = Implementor(100);
        while let Ok(msg) = rx1.recv_async().await {
            eprintln!("> {}", msg);
            let x: MyIfaceEnum = serde_json::from_str(&msg).unwrap();
            x.try_call_mut(&mut o, &tx2).await.unwrap();
        }
    });

    let c = MyRpcClient {
        pending_replies: Arc::new(Mutex::new(slab::Slab::new())),
    };
    let c2 = c.clone();

    tokio::spawn(async move {
        while let Ok(x) = rx2.recv_async().await {
            eprintln!("< {}", x);
            c2.handle_reply(x).await;
        }
    });

    let tx1_ = tx1.clone();
    let c_ = c.clone();
    tokio::spawn(async move {
        let p = MyIfaceProxy::<_, _, _>(
            move |msg| {
                let tx1_ = tx1_.clone();
                async move { tx1_.send_async(serde_json::to_string(&msg).unwrap()).await }
            },
            c_,
        );
        p.try_sleep_making_caller_wait_for_it(5).await.unwrap().unwrap();
        dbg!(p.try_get().await.unwrap().unwrap());
        p.try_sleep_making_caller_wait_for_it(5).await.unwrap().unwrap();
        dbg!(p.try_get().await.unwrap().unwrap());
        p.try_sleep_making_caller_wait_for_it(5).await.unwrap().unwrap();
        dbg!(p.try_get().await.unwrap().unwrap());
        p.try_sleep_making_caller_wait_for_it(5).await.unwrap().unwrap();
        dbg!(p.try_get().await.unwrap().unwrap());
    });

    let p = MyIfaceProxy::<_, _, _>(
        move |msg| {
            let tx1 = tx1.clone();
            async move { tx1.send_async(serde_json::to_string(&msg).unwrap()).await }
        },
        c,
    );
    p.try_addone().await.unwrap();
    p.try_addone().await.unwrap();
    dbg!(p.try_get().await.unwrap().unwrap());
    p.try_double().await.unwrap();
    p.try_sleep_without_caller_waiting_for_it(20).await.unwrap();
    dbg!(p.try_get().await.unwrap().unwrap());
    p.try_divide(3).await.unwrap();
    eprintln!(
        "{}",
        p.try_format("[[[".to_owned(), "]]]".to_owned())
            .await
            .unwrap()
            .unwrap()
    );
}
