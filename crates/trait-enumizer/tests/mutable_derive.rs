#[trait_enumizer::enumizer(call_mut(),mut_proxy(infallible_impl))]
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

#[test]
fn test() {
    let mut o = Implementor {};
    let mut p = MyIfaceProxyMut::<std::convert::Infallible,_>(|c| Ok(c.call_mut(&mut o)));
    p.foo();
    p.bar(4);
}
