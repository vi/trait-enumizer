#![cfg(feature="flume")]

use trait_enumizer::flume_class;

struct Qqq {}

impl Qqq {
    fn foo(&self) -> String {
        dbg!("foo");
        "qqq".to_owned()
    }

    async fn bar(&self, x: i32) -> i32 {
        dbg!("bar", x);
        333
    }

    async fn baz(&self, y: String, z: Vec<u8>) {
        dbg!("baz", y, z);
    }
}

// Begin of the part which is supposed to be auto-generated

enum QqqEnum {
    Foo { ret: flume_class!(Sender<String>)},
    Bar { x: i32, ret: flume_class!(Sender<i32>) },
    Baz { y: String, z: Vec<u8> },
}

impl QqqEnum {
    async fn try_call(self, o: &Qqq) -> Result<(), flume_class!(SendError) > {
        match self {
            QqqEnum::Foo { ret} => flume_class!(send_async::<String>(ret, o.foo())),
            QqqEnum::Bar { x , ret} => flume_class!(send_async::<i32>(ret, o.bar(x).await)),
            QqqEnum::Baz { y, z } => Ok(o.baz(y, z).await),
        }
    }
}

struct QqqUsualProxy<E, F: Fn(QqqEnum) -> Result<(), E>>(F);
impl<E, F: Fn(QqqEnum) -> Result<(), E>> QqqUsualProxy<E, F> {
    fn try_foo(&self) -> Result<Result<String, flume_class!(RecvError)>, E>{
        let (tx, rx) = flume_class!(create::<String>());
        self.0(QqqEnum::Foo { ret: tx })?;
        Ok(flume_class!(recv::<i32>(rx)))
    }

    fn try_bar(&self, x: i32) -> Result<Result<i32, flume_class!(RecvError)>, E> {
        let (tx, rx) = flume_class!(create::<i32>());
        self.0(QqqEnum::Bar { x, ret: tx })?;
        Ok(flume_class!(recv::<i32>(rx)))
    }

    fn try_baz(&self, y: String, z: Vec<u8>) -> Result<(), E> {
        self.0(QqqEnum::Baz { y, z })
    }
}
struct QqqAsyncProxy<E, F: Fn(QqqEnum) -> Fu, Fu: std::future::Future<Output = Result<(), E>>>(F);
impl<E, F: Fn(QqqEnum) -> Fu, Fu: std::future::Future<Output = Result<(), E>>> QqqAsyncProxy<E, F, Fu> {
    async fn try_foo(&self) -> Result<Result<String, flume_class!(RecvError)>, E> {
        let (tx, rx) = flume_class!(create::<String>());
        self.0(QqqEnum::Foo { ret: tx }).await?;
        Ok(flume_class!(recv_async::<i32>(rx)))
    }

    async fn try_bar(&self, x: i32) -> Result<Result<i32, flume_class!(RecvError)>, E> {
        let (tx, rx) = flume_class!(create::<i32>());
        self.0(QqqEnum::Bar { x, ret: tx }).await?;
        Ok(flume_class!(recv_async::<i32>(rx)))
    }

    async fn try_baz(&self, y: String, z: Vec<u8>) -> Result<(), E> {
        self.0(QqqEnum::Baz { y, z }).await
    }
}

// End of the part which is supposed to be auto-generated

#[test]
fn async_to_async() {
    let o = Qqq {};
    let p = QqqAsyncProxy::<std::convert::Infallible, _, _>(|c: QqqEnum| async { Ok(c.try_call(&o).await.unwrap()) });

    dbg!(futures::executor::block_on(p.try_foo()).unwrap().unwrap());
    dbg!(futures::executor::block_on(p.try_bar(4)).unwrap().unwrap());
    dbg!(futures::executor::block_on(p.try_baz("qqq".to_owned(), vec![])).unwrap());
}


#[test]
fn usual_to_async() {
    let o = Qqq {};
    let p = QqqUsualProxy::<std::convert::Infallible, _>(|c: QqqEnum| Ok(futures::executor::block_on(c.try_call(&o)).unwrap()));

    dbg!(p.try_foo().unwrap().unwrap());
    dbg!(p.try_bar(4).unwrap().unwrap());
    dbg!(p.try_baz("qqq".to_owned(), vec![]).unwrap());
}
