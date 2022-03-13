mod inner {
    #[trait_enumizer::enumizer(
        name=TheEnum,
        pub_crate,
        call_fn(name=call_mut, ref_mut, allow_panic),
        proxy(Fn,unwrapping_impl,name=TheCaller,resultified_trait=ITheCaller),
        enum_attr[derive(serde_derive::Serialize)]
    )]
    pub trait MyIface {
        fn increment(&mut self);
        fn increment2(&mut self, #[enumizer_enum_attr[serde(rename="www")]] x : i32);
        #[enumizer_enum_attr[serde(rename="qqq")]]
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

use inner::{MyIface, TheEnum, TheCaller};

#[test]
fn test() {
    let (tx,rx) = flume::bounded::<TheEnum>(1);
    std::thread::spawn(move || {
        let mut o = inner::Implementor(100);
        for msg in rx {
            eprintln!("{}", serde_json::ser::to_string(&msg).unwrap());
            msg.call_mut(&mut o);
        }
    });
    let mut p = TheCaller::<_, _>(|c| tx.send(c));
    dbg!(p.reset());
    dbg!(p.increment2(4));
    dbg!(p.print());
    //p.gulp();
}
