struct Qqq;

#[trait_enumizer::enumizer(inherent_impl, name=QqqEnum, call_fn(ref,name=call), proxy(Fn,name=QqqProxy))]
impl Qqq {
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
fn test() {
    let o = Qqq;
    let p = QqqProxy::<std::convert::Infallible,_>(|c : QqqEnum| Ok(c.call(&o)));
    p.try_foo().unwrap();
    p.try_bar(4).unwrap();
}
