struct Qqq {}

impl Qqq {
    fn foo(&self) {
        dbg!("foo");
    }

    async fn bar(&self, x: i32) {
        dbg!("bar", x);
    }

    async fn baz(&self, y: String, z: Vec<u8>) {
        dbg!("baz", y, z);
    }
}

// Begin of the part which is supposed to be auto-generated

enum QqqEnum {
    Foo,
    Bar { x: i32 },
    Baz { y: String, z: Vec<u8> },
}

impl QqqEnum {
    async fn call(self, o: &Qqq) {
        match self {
            QqqEnum::Foo => o.foo(),
            QqqEnum::Bar { x } => o.bar(x).await,
            QqqEnum::Baz { y, z } => o.baz(y, z).await,
        }
    }
    #[allow(unused)]
    async fn call_mut(self, o: &mut Qqq) {
        match self {
            QqqEnum::Foo => o.foo(),
            QqqEnum::Bar { x } => o.bar(x).await,
            QqqEnum::Baz { y, z } => o.baz(y, z).await,
        }
    }

    #[allow(unused)]
    async fn call_once(self, o: Qqq) {
        match self {
            QqqEnum::Foo => o.foo(),
            QqqEnum::Bar { x } => o.bar(x).await,
            QqqEnum::Baz { y, z } => o.baz(y, z).await,
        }
    }
}

struct QqqUsualProxy<E, F: Fn(QqqEnum) -> Result<(), E>>(F);
impl<E, F: Fn(QqqEnum) -> Result<(), E>> QqqUsualProxy<E, F> {
    fn try_foo(&self) -> Result<(), E> {
        self.0(QqqEnum::Foo)
    }

    fn try_bar(&self, x: i32) -> Result<(), E> {
        self.0(QqqEnum::Bar { x })
    }

    fn try_baz(&self, y: String, z: Vec<u8>) -> Result<(), E> {
        self.0(QqqEnum::Baz { y, z })
    }
}
struct QqqAsyncProxy<E, F: Fn(QqqEnum) -> Fu, Fu: std::future::Future<Output = Result<(), E>>>(F);
impl<E, F: Fn(QqqEnum) -> Fu, Fu: std::future::Future<Output = Result<(), E>>> QqqAsyncProxy<E, F, Fu> {
    async fn try_foo(&self) -> Result<(), E> {
        self.0(QqqEnum::Foo).await
    }

    async fn try_bar(&self, x: i32) -> Result<(), E> {
        self.0(QqqEnum::Bar { x }).await
    }

    async fn try_baz(&self, y: String, z: Vec<u8>) -> Result<(), E> {
        self.0(QqqEnum::Baz { y, z }).await
    }
}

// End of the part which is supposed to be auto-generated

#[test]
fn async_to_async() {
    let o = Qqq {};
    let p = QqqAsyncProxy::<std::convert::Infallible, _, _>(|c: QqqEnum| async { Ok(c.call(&o).await) });

    futures::executor::block_on(p.try_foo()).unwrap();
    futures::executor::block_on(p.try_bar(4)).unwrap();
    futures::executor::block_on(p.try_baz("qqq".to_owned(), vec![])).unwrap();
}


#[test]
fn usual_to_async() {
    let o = Qqq {};
    let p = QqqUsualProxy::<std::convert::Infallible, _>(|c: QqqEnum| Ok(futures::executor::block_on(c.call(&o))));

    p.try_foo().unwrap();
    p.try_bar(4).unwrap();
    p.try_baz("qqq".to_owned(), vec![]).unwrap();
}
