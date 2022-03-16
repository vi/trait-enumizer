#![cfg(feature="flume")]

use trait_enumizer::flume_class;

struct Qqq {}

#[trait_enumizer::enumizer(
    name=QqqEnum,
    inherent_impl,
    returnval=flume_class,
    call_fn(ref,name=try_call,async),
    proxy(Fn,name=QqqUsualProxy),
    proxy(Fn,name=QqqAsyncProxy,async),
)]
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
