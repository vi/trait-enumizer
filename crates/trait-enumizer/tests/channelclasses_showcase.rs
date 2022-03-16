#![cfg(feature="flume")]
#![cfg(feature="crossbeam-channel")]
#![cfg(feature="tokio")]
#![cfg(feature="catty")]
#![cfg(feature="futures")]

use trait_enumizer::flume_class;
use trait_enumizer::crossbeam_class;
use trait_enumizer::tokio_oneshot_class;
use trait_enumizer::catty_class;
use trait_enumizer::futures_oneshot_class;

struct Qqq {}

#[trait_enumizer::enumizer(
    name=WithFlume,
    inherent_impl,
    returnval=flume_class,
    call_fn(ref,name=try_call),
    proxy(Fn,name=FlumeSyncProxy),
    proxy(Fn,name=FlumeAsyncProxy,async),
)]
#[trait_enumizer::enumizer(
    name=WithCrossbeam,
    inherent_impl,
    returnval=crossbeam_class,
    call_fn(ref,name=try_call),
    proxy(Fn,name=CrossbeamSyncProxy),
)]
#[trait_enumizer::enumizer(
    name=WithTokioOneshot,
    inherent_impl,
    returnval=tokio_oneshot_class,
    call_fn(ref,name=try_call),
    proxy(Fn,name=TokioOneshotAsyncProxy,async),
    proxy(Fn,name=TokioOneshotSyncProxy),
)]
#[trait_enumizer::enumizer(
    name=WithCatty,
    inherent_impl,
    returnval=catty_class,
    call_fn(ref,name=try_call),
    proxy(Fn,name=CattyProxy,async),
)]
#[trait_enumizer::enumizer(
    name=WithFuturesOneshot,
    inherent_impl,
    returnval=futures_oneshot_class,
    call_fn(ref,name=try_call),
    proxy(Fn,name=FuturesOneshotProxy,async),
)]
impl Qqq {
    fn foo(&self) -> String {
        dbg!("foo");
        "qqq".to_owned()
    }
}

#[test]
fn flume_async() {
    let o = Qqq {};
    let p = FlumeAsyncProxy::<std::convert::Infallible, _, _>(|c: WithFlume| async { Ok(c.try_call(&o).unwrap()) });

    dbg!(futures::executor::block_on(p.try_foo()).unwrap().unwrap());
}


#[test]
fn flume_sync() {
    let o = Qqq {};
    let p = FlumeSyncProxy::<std::convert::Infallible, _>(|c: WithFlume| Ok(c.try_call(&o).unwrap()));

    dbg!(p.try_foo().unwrap().unwrap());
}

#[test]
fn crossbeam_sync() {
    let o = Qqq {};
    let p = CrossbeamSyncProxy::<std::convert::Infallible, _>(|c: WithCrossbeam| Ok(c.try_call(&o).unwrap()));

    dbg!(p.try_foo().unwrap().unwrap());
}


#[tokio::test]
async fn tokio_async() {
    let o = Qqq {};
    let p = TokioOneshotAsyncProxy::<std::convert::Infallible, _, _>(|c: WithTokioOneshot| async { Ok(c.try_call(&o).unwrap()) });

    dbg!(p.try_foo().await.unwrap().unwrap());
}

#[test]
fn tokio_sync() {
    let o = Qqq {};
    let p = TokioOneshotSyncProxy::<std::convert::Infallible, _>(|c: WithTokioOneshot| Ok(c.try_call(&o).unwrap()));

    dbg!(p.try_foo().unwrap().unwrap());
}

#[tokio::test]
async fn catty_async() {
    let o = Qqq {};
    let p = CattyProxy::<std::convert::Infallible, _, _>(|c: WithCatty| async { Ok(c.try_call(&o).unwrap()) });

    dbg!(p.try_foo().await.unwrap().unwrap());
}


#[tokio::test]
async fn futures_oneshot() {
    let o = Qqq {};
    let p = FuturesOneshotProxy::<std::convert::Infallible, _, _>(|c: WithFuturesOneshot| async { Ok(c.try_call(&o).unwrap()) });

    dbg!(p.try_foo().await.unwrap().unwrap());
}
