use std::borrow::Borrow;

trait MyIface {
    fn primitive(&self, x: i32);
    fn by_value(&self, x: String);
    fn by_ref(&self, y: &str);
    fn by_ref2(&self, y: &[u8]);
}

struct Implementor {}

impl MyIface for Implementor {
    fn primitive(&self, x: i32) {
        dbg!(x);
    }

    fn by_value(&self, x: String) {
        dbg!(x);
    }

    fn by_ref(&self, y: &str) {
        dbg!(y);
    }
    fn by_ref2(&self, y: &[u8]) {
        dbg!(y);
    }
}

// Begin of the part which is supposed to be auto-generated

enum MyIfaceEnum {
    Primitive { x : i32 },
    ByValue { x: String },
    ByRef { y: <str as ToOwned>::Owned },
    ByRef2 { y: <[u8] as ToOwned>::Owned },
}

impl MyIfaceEnum {
    fn call<I: MyIface>(self, o: &I) {
        match self {
            MyIfaceEnum::Primitive {x}  => o.primitive(x),
            MyIfaceEnum::ByValue { x } => o.by_value(x),
            MyIfaceEnum::ByRef { y } => o.by_ref(Borrow::borrow(&y)),
            MyIfaceEnum::ByRef2 { y } => o.by_ref2(Borrow::borrow(&y)),
        }
    }
    #[allow(unused)]
    fn call_mut<I: MyIface>(self, o: &mut I) {
        match self {
            MyIfaceEnum::Primitive {x}  => o.primitive(x),
            MyIfaceEnum::ByValue { x } => o.by_value(x),
            MyIfaceEnum::ByRef { y } => o.by_ref(Borrow::borrow(&y)),
            MyIfaceEnum::ByRef2 { y } => o.by_ref2(Borrow::borrow(&y)),
        }
    }

    #[allow(unused)]
    fn call_once<I: MyIface>(self, o: I) {
        match self {
            MyIfaceEnum::Primitive {x}  => o.primitive(x),
            MyIfaceEnum::ByValue { x } => o.by_value(x),
            MyIfaceEnum::ByRef { y } => o.by_ref(Borrow::borrow(&y)),
            MyIfaceEnum::ByRef2 { y } => o.by_ref2(Borrow::borrow(&y)),
        }
    }
}
trait MyIfaceResultified<E> {
    fn try_primitive(&self, x: i32) -> Result<(), E>;
    fn try_by_value(&self, x: String) -> Result<(), E>;
    fn try_by_ref(&self, y: &str) -> Result<(), E>;
    fn try_by_ref2(&self, y: &[u8]) -> Result<(), E>;
}

impl<R:MyIfaceResultified<std::convert::Infallible>> MyIface for R {
    fn primitive(&self, x: i32) {
        MyIfaceResultified::try_primitive(self, x).unwrap()
    }

    fn by_value(&self, x: String) {
        MyIfaceResultified::try_by_value(self, x).unwrap()
    }

    fn by_ref(&self, y: &str) {
        MyIfaceResultified::try_by_ref(self, y).unwrap()
    }

    fn by_ref2(&self, y: &[u8]) {
        MyIfaceResultified::try_by_ref2(self, y).unwrap()
    }
}

struct MyIfaceProxy<E, F: Fn(MyIfaceEnum)-> Result<(), E> > (F);
impl<E, F: Fn(MyIfaceEnum) -> Result<(), E>> MyIfaceResultified<E> for MyIfaceProxy<E, F> {
    fn try_primitive(&self, x: i32) -> Result<(), E> {
        (self.0)(MyIfaceEnum::Primitive { x } )
    }

    fn try_by_value(&self, x: String) -> Result<(), E> {
        (self.0)(MyIfaceEnum::ByValue { x } )
    }

    fn try_by_ref(&self, y: &str) -> Result<(), E> {
        (self.0)(MyIfaceEnum::ByRef { y: ToOwned::to_owned(y) } )
    }

    fn try_by_ref2(&self, y: &[u8]) -> Result<(), E> {
        (self.0)(MyIfaceEnum::ByRef2 { y: ToOwned::to_owned(y) } )
    }
}

// End of the part which is supposed to be auto-generated


#[test]
fn test() {
    let o = Implementor {};
    let p = MyIfaceProxy::<std::convert::Infallible,_>(|c| Ok(c.call(&o)));
    p.primitive(3);
    p.by_value("owned".to_owned());
    p.by_ref("by_ref");
    p.by_ref2(b"by_ref2");
}
