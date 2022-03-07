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
        x*x+1
    }

    fn baz(&self, y: String, z: Vec<u8>) {
        dbg!("baz", y, z);
    }
}

// Begin of the part which is supposed to be auto-generated

enum MyIfaceEnum {
    Foo { ret: flume::Sender<String> },
    Bar { x: i32, ret: flume::Sender<i32> },
    Baz { y: String, z: Vec<u8> },
}

impl MyIfaceEnum {
    fn try_call<I: MyIface>(self, o: &I) -> Result<(), &'static str> {
        match self {
            MyIfaceEnum::Foo {ret} => Ok(ret.send(o.foo()).map_err(|_|"Failed to return value though enumizer channel")?),
            MyIfaceEnum::Bar { x, ret } => Ok(ret.send(o.bar(x)).map_err(|_|"Failed to return value though enumizer channel")?),
            MyIfaceEnum::Baz { y, z } => Ok(o.baz(y, z)),
        }
    }
}
trait MyIfaceResultified<E1> {
    fn try_foo(&self) -> Result<Result<String, flume::RecvError>, E1>;
    fn try_bar(&self, x: i32) -> Result<Result<i32, flume::RecvError>, E1>;
    fn try_baz(&self, y: String, z: Vec<u8>) -> Result<(), E1>;
}

impl<R:MyIfaceResultified<std::convert::Infallible>> MyIface for R {
    fn foo(&self) -> String {
        R::try_foo(self).unwrap().unwrap()
    }

    fn bar(&self, x: i32) -> i32 {
        R::try_bar(self, x).unwrap().unwrap()
    }

    fn baz(&self, y: String, z: Vec<u8>) {
        R::try_baz(self,y,z).unwrap()
    }
}

struct MyIfaceProxy<E1, F1: Fn(MyIfaceEnum)-> Result<(), E1>> (F1);
impl<E1, F1: Fn(MyIfaceEnum)-> Result<(), E1> > MyIfaceResultified<E1> for MyIfaceProxy<E1, F1> {
    fn try_foo(&self) -> Result<Result<String, flume::RecvError>, E1> {
        let (tx, rx) = flume::bounded(1);
        self.0(MyIfaceEnum::Foo{ret:tx})?;
        Ok(rx.recv())
    }

    fn try_bar(&self, x: i32) -> Result<Result<i32, flume::RecvError>, E1> {
        let (tx, rx) = flume::bounded(1);
        self.0(MyIfaceEnum::Bar { x, ret: tx })?;
        Ok(rx.recv())
    }

    fn try_baz(&self, y: String, z: Vec<u8>) -> Result<(), E1> {
        self.0(MyIfaceEnum::Baz { y, z })
    }
}

// End of the part which is supposed to be auto-generated

#[test]
fn simple() {
    let o = Implementor {};
    let p = MyIfaceProxy::<_,_>(|c| c.try_call(&o));
    dbg!(p.try_foo().unwrap().unwrap());
    dbg!(p.try_bar(4).unwrap().unwrap());
}
