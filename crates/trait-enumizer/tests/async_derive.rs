struct Qqq {}

#[trait_enumizer::enumizer(
    name=QqqEnum,
    inherent_impl,
    call_fn(name=call, async, ref)
    proxy(Fn, name=QqqAsyncProxy, async)
    proxy(Fn, name=QqqUsualProxy, no_async)
)]
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
