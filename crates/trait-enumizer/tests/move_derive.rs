#[trait_enumizer::enumizer(name=MyIfaceEnum,call_fn(once,name=call_once),proxy(FnOnce,name=MyIfaceProxyOnce,resultified_trait=MyIfaceResultifiedOnce,infallible_impl))]
trait MyIface {
    fn foo(self);
    fn bar(self, x: i32);
    fn baz(self, y: String, z: Vec<u8>);
}

struct Implementor {}

impl MyIface for Implementor {
    fn foo(self) {
        dbg!("foo");
    }

    fn bar(self, x: i32) {
        dbg!("bar", x);
    }

    fn baz(self, y: String, z: Vec<u8>) {
        dbg!("baz", y, z);
    }
}

#[test]
fn test() {
    let o1 = Implementor {};
    let p1 = MyIfaceProxyOnce::<std::convert::Infallible,_>(move |c| Ok(c.call_once(o1)));
    let o2 = Implementor {};
    let p2 = MyIfaceProxyOnce::<std::convert::Infallible,_>(move |c| Ok(c.call_once(o2)));
    p1.foo();
    p2.bar(4);
}
