trait MyIface {
    fn foo(&mut self);
    fn bar(&mut self, x: i32);
    fn baz(&mut self, y: String, z: Vec<u8>);
}

struct Implementor {}

impl MyIface for Implementor {
    fn foo(&mut self) {
        dbg!("foo");
    }

    fn bar(&mut self, x: i32) {
        dbg!("bar", x);
    }

    fn baz(&mut self, y: String, z: Vec<u8>) {
        dbg!("baz", y, z);
    }
}

// Begin of the part which is supposed to be auto-generated

enum MyIfaceEnum {
    Foo,
    Bar { x: i32 },
    Baz { y: String, z: Vec<u8> },
}

impl MyIfaceEnum {
    fn call_mut<I: MyIface>(self, o: &mut I) {
        match self {
            MyIfaceEnum::Foo => o.foo(),
            MyIfaceEnum::Bar { x } => o.bar(x),
            MyIfaceEnum::Baz { y, z } => o.baz(y, z),
        }
    }
}
trait MyIfaceResultified<E> {
    fn try_foo(&mut self) -> Result<(), E>;
    fn try_bar(&mut self, x: i32) -> Result<(), E>;
    fn try_baz(&mut self, y: String, z: Vec<u8>) -> Result<(), E>;
}

impl<R:MyIfaceResultified<std::convert::Infallible>> MyIface for R {
    fn foo(&mut self) {
        R::try_foo(self).unwrap()
    }

    fn bar(&mut self, x: i32) {
        R::try_bar(self, x).unwrap()
    }

    fn baz(&mut self, y: String, z: Vec<u8>) {
        R::try_baz(self,y,z).unwrap()
    }
}

struct MyIfaceProxy<E, F: FnMut(MyIfaceEnum)-> Result<(), E> > (F);
impl<E, F: FnMut(MyIfaceEnum) -> Result<(), E>> MyIfaceResultified<E> for MyIfaceProxy<E, F> {
    fn try_foo(&mut self) -> Result<(), E> {
        self.0(MyIfaceEnum::Foo)
    }

    fn try_bar(&mut self, x: i32) -> Result<(), E> {
        self.0(MyIfaceEnum::Bar { x })
    }

    fn try_baz(&mut self, y: String, z: Vec<u8>) -> Result<(), E> {
        self.0(MyIfaceEnum::Baz { y, z })
    }
}

// End of the part which is supposed to be auto-generated

#[test]
fn test() {
    let mut o = Implementor {};
    let mut p = MyIfaceProxy::<std::convert::Infallible,_>(|c| Ok(c.call_mut(&mut o)));
    p.foo();
    p.bar(4);
}
