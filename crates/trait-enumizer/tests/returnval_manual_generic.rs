#![cfg(feature="returnval")]
#![feature(generic_associated_types)]

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

enum MyIfaceEnum<CC: SyncChannelClass> {
    Foo { ret: CC::Sender<String> },
    Bar { x: i32, ret: CC::Sender<i32> },
    Baz { y: String, z: Vec<u8> },
}

impl<CC: SyncChannelClass> MyIfaceEnum<CC> {
    fn try_call<I: MyIface>(self, o: &I) -> Result<(), CC::SendError> {
        match self {
            MyIfaceEnum::Foo { ret } => Ok(CC::send(ret, o.foo())?),
            MyIfaceEnum::Bar { x, ret } => Ok(CC::send(ret, o.bar(x))?),
            MyIfaceEnum::Baz { y, z } => Ok(o.baz(y, z)),
        }
    }
}
trait MyIfaceResultified<E, CC: SyncChannelClass> {
    fn try_foo(&self) -> Result<Result<String, CC::RecvError>, E>;
    fn try_bar(&self, x: i32) -> Result<Result<i32, CC::RecvError>, E>;
    fn try_baz(&self, y: String, z: Vec<u8>) -> Result<(), E>;
}

struct MyIfaceProxy<CC, E, F>(F, CC)
where
    CC: SyncChannelClass,
    F: Fn(MyIfaceEnum<CC>) -> Result<(), E>;

impl<CC, E, F> MyIfaceResultified<E, CC> for MyIfaceProxy<CC, E, F>
where
    CC: SyncChannelClass,
    F: Fn(MyIfaceEnum<CC>) -> Result<(), E>,
{
    fn try_foo(&self) -> Result<Result<String, CC::RecvError>, E> {
        let (tx, rx) = self.1.create();
        self.0(MyIfaceEnum::Foo { ret: tx })?;
        Ok(CC::recv(rx))
    }

    fn try_bar(&self, x: i32) -> Result<Result<i32, CC::RecvError>, E> {
        let (tx, rx) = self.1.create();
        self.0(MyIfaceEnum::Bar { x, ret: tx })?;
        Ok(CC::recv(rx))
    }

    fn try_baz(&self, y: String, z: Vec<u8>) -> Result<(), E> {
        self.0(MyIfaceEnum::Baz { y, z })
    }
}

impl<CC, E, F> MyIface for MyIfaceProxy<CC, E, F>
where
    CC: SyncChannelClass,
    F: Fn(MyIfaceEnum<CC>) -> Result<(), E>,
    E :  std::fmt::Debug,
    CC::RecvError : std::fmt::Debug,
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

use trait_enumizer::{SyncChannelClass, FlumeChannelClass};

#[test]
fn simple() {
    let o = Implementor {};
    let p = MyIfaceProxy::<_, _, _>(|c| c.try_call(&o), FlumeChannelClass);
    dbg!(p.foo());
    dbg!(p.bar(4));
}


#[test]
fn threaded() {
    let (tx,rx) = flume::bounded::<MyIfaceEnum<FlumeChannelClass>>(1);
    std::thread::spawn(move || {
        let o = Implementor {};
        for msg in rx {
            msg.try_call(&o).unwrap();
        }
    });
    let p = MyIfaceProxy::<_, _, _>(|c| tx.send(c), FlumeChannelClass);
    dbg!(p.foo());
    dbg!(p.bar(4));
}
