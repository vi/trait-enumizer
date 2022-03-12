#[trait_enumizer::enumizer(call,ref_proxy(infallible_impl))]
trait MyIface {
    fn primitive(&self, x: i32);
    fn by_value(&self, x: String);
    fn by_ref(&self, #[enumizer_to_owned] y: &str);
    fn by_ref2(&self, #[enumizer_to_owned] y: &[u8]);
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

#[test]
fn test() {
    let o = Implementor {};
    let p = MyIfaceProxy::<std::convert::Infallible,_>(|c| Ok(c.call(&o)));
    p.primitive(3);
    p.by_value("owned".to_owned());
    p.by_ref("by_ref");
    p.by_ref2(b"by_ref2");
}
