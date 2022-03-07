#[trait_enumizer::enumizer(call_mut(allow_panic),ref_proxy(unwrapping_impl))]
trait MyIface {
    fn increment(&mut self);
    fn print(&self);
    fn reset(&mut self);
    fn gulp(self);
}

struct Implementor(i32);

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
}

#[test]
fn test() {
    let (tx,rx) = flume::bounded::<MyIfaceEnum>(1);
    std::thread::spawn(move || {
        let mut o = Implementor(100);
        for msg in rx {
            msg.call_mut(&mut o);
        }
    });
    let mut p = MyIfaceProxy::<_, _>(|c| tx.send(c));
    dbg!(p.reset());
    dbg!(p.increment());
    dbg!(p.print());
    //p.gulp();
}
