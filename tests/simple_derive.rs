#[trait_enumizer::enumizer]
trait MyIface {
    fn foo(&self);
    fn bar(&self, x: i32);
    fn baz(&self, y: String, z: Vec<u8>);
}

struct Implementor {}

impl MyIface for Implementor {
    fn foo(&self) {
        dbg!("foo");
    }

    fn bar(&self, x: i32) {
        dbg!("bar", x);
    }

    fn baz(&self, y: String, z: Vec<u8>) {
        dbg!("baz", y, z);
    }
}

#[test]
fn simple() {
    let o = Implementor {};
    let p = MyIfaceProxy::<std::convert::Infallible,_>(|c| Ok(c.call(&o)));
    p.foo();
    p.bar(4);
}
