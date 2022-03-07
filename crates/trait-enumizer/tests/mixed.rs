mod inner {
    #[trait_enumizer::enumizer(pub_crate, call_mut(allow_panic),ref_proxy(unwrapping_impl))]
    pub trait MyIface {
        fn increment(&mut self);
        fn increment2(&mut self, x : i32);
        fn print(&self);
        fn reset(&mut self);
        fn gulp(self);
    }

    pub struct Implementor(pub i32);

    impl MyIface for Implementor {
        fn increment(&mut self) {
            self.0 += 1;
        }

        fn print(&self) {
            dbg!(self.0);
        }

        fn reset(&mut self) {
            self.0 = 0;
        }

        fn gulp(self) {
            dbg!("gulp!");
        }

        fn increment2(&mut self, x : i32) {
            self.0 += x;
        }
    }
}

use inner::{MyIface, MyIfaceEnum, MyIfaceProxy};

#[test]
fn test() {
    let (tx,rx) = flume::bounded::<MyIfaceEnum>(1);
    std::thread::spawn(move || {
        let mut o = inner::Implementor(100);
        for msg in rx {
            msg.call_mut(&mut o);
        }
    });
    let mut p = MyIfaceProxy::<_, _>(|c| tx.send(c));
    dbg!(p.reset());
    dbg!(p.increment2(4));
    dbg!(p.print());
    //p.gulp();
}
