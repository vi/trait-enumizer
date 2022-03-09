#![cfg(feature="flume")]

use trait_enumizer::flume_class;

trait MyIface {
    fn foo(&self) -> String;
    fn bar(&self, x: i32) -> i32;
    fn baz(&self, y: String, z: Vec<u8>);
}

struct Implementor {}

impl MyIface for Implementor {
    fn foo(&self) -> String {
        dbg!("foo");
        "qqq".to_owned()
    }

    fn bar(&self, x: i32) -> i32 {
        dbg!("bar", x);
        x * x + 1
    }

    fn baz(&self, y: String, z: Vec<u8>) {
        dbg!("baz", y, z);
    }
}

// Begin of the part which is supposed to be auto-generated

enum MyIfaceEnum {
    Foo { ret: flume_class!(Sender<String>) },
    Bar { x: i32, ret: flume_class!(Sender<i32>) },
    Baz { y: String, z: Vec<u8> },
}

impl MyIfaceEnum {
    fn try_call<I: MyIface>(self, o: &I) -> Result<(), flume_class!(SendError)> {
        match self {
            MyIfaceEnum::Foo { ret } => Ok(flume_class!(send::<String>(ret, o.foo()))?),
            MyIfaceEnum::Bar { x, ret } => Ok(flume_class!(send::<i32>(ret, o.bar(x)))?),
            MyIfaceEnum::Baz { y, z } => Ok(o.baz(y, z)),
        }
    }
}
trait MyIfaceResultified<E> {
    fn try_foo(&self) -> Result<Result<String, flume_class!(RecvError)>, E>;
    fn try_bar(&self, x: i32) -> Result<Result<i32, flume_class!(RecvError)>, E>;
    fn try_baz(&self, y: String, z: Vec<u8>) -> Result<(), E>;
}

struct MyIfaceProxy<E, F>(F)
where
    F: Fn(MyIfaceEnum) -> Result<(), E>;

impl<E, F> MyIfaceResultified<E> for MyIfaceProxy<E, F>
where
    F: Fn(MyIfaceEnum) -> Result<(), E>,
{
    fn try_foo(&self) -> Result<Result<String, flume_class!(RecvError)>, E> {
        let (tx, rx) = flume_class!(create::<String>());
        self.0(MyIfaceEnum::Foo { ret: tx })?;
        Ok(flume_class!(recv::<i32>(rx)))
    }

    fn try_bar(&self, x: i32) -> Result<Result<i32, flume_class!(RecvError)>, E> {
        let (tx, rx) = flume_class!(create::<i32>());
        self.0(MyIfaceEnum::Bar { x, ret: tx })?;
        Ok(flume_class!(recv::<i32>(rx)))
    }

    fn try_baz(&self, y: String, z: Vec<u8>) -> Result<(), E> {
        self.0(MyIfaceEnum::Baz { y, z })
    }
}

impl<E, F> MyIface for MyIfaceProxy<E, F>
where
    F: Fn(MyIfaceEnum) -> Result<(), E>,
    E :  std::fmt::Debug,
    flume_class!(RecvError) : std::fmt::Debug,
{
    fn foo(&self) -> String {
        MyIfaceResultified::try_foo(self).unwrap().unwrap()
    }

    fn bar(&self, x: i32) -> i32 {
        MyIfaceResultified::try_bar(self, x).unwrap().unwrap()
    }

    fn baz(&self, y: String, z: Vec<u8>) {
        MyIfaceResultified::try_baz(self, y, z).unwrap()
    }
}

// End of the part which is supposed to be auto-generated


#[test]
fn simple() {
    let o = Implementor {};
    let p = MyIfaceProxy::<_, _>(move |c| c.try_call(&o));
    dbg!(p.foo());
    dbg!(p.bar(4));
}


#[test]
fn threaded() {
    let (tx,rx) = flume::bounded::<MyIfaceEnum>(1);
    std::thread::spawn(move || {
        let o = Implementor {};
        for msg in rx {
            msg.try_call(&o).unwrap();
        }
    });
    let p = MyIfaceProxy::<_, _>(|c| tx.send(c));
    dbg!(p.foo());
    dbg!(p.bar(4));
}
