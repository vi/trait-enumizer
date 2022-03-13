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

struct QqqProxy<E, Fu: std::future::Future<Output = Result<(), E>>, F: Fn(QqqEnum) -> Fu>(F);
impl<E, Fu: std::future::Future<Output = Result<(), E>>, F: Fn(QqqEnum) -> Fu> QqqProxy<E, Fu, F> {
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
fn simple() {
    let o = Qqq {};
    let p = QqqProxy::<std::convert::Infallible, _, _>(|c: QqqEnum| async { Ok(c.call(&o).await) });

    futures::executor::block_on(p.try_foo()).unwrap();
    futures::executor::block_on(p.try_bar(4)).unwrap();
    futures::executor::block_on(p.try_baz("qqq".to_owned(), vec![])).unwrap();
}
