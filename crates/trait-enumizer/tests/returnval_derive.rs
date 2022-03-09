#![cfg(feature="returnval")]
#![feature(generic_associated_types)]

use trait_enumizer::flume_class;

#[trait_enumizer::enumizer(returnval=flume_class,call,ref_proxy(unwrapping_impl))]
trait MyIface {
    fn foo(&self) -> String;
    fn bar(&self, x: i32) -> i32;
    fn baz(&self, y: String, z: Vec<u8>);
}

struct Implementor {}

impl MyIface for Implementor {
    fn foo(&self) -> String {
        dbg!("foo");
        "qqq".to_owned()
    }

    fn bar(&self, x: i32) -> i32 {
        dbg!("bar", x);
        x * x + 1
    }

    fn baz(&self, y: String, z: Vec<u8>) {
        dbg!("baz", y, z);
    }
}

#[test]
fn simple() {
    let o = Implementor {};
    let p = MyIfaceProxy::<_, _>(|c| c.try_call(&o));
    dbg!(p.foo());
    dbg!(p.bar(4));
}


#[test]
fn threaded() {
    let (tx,rx) = flume::bounded::<MyIfaceEnum>(1);
    std::thread::spawn(move || {
        let o = Implementor {};
        for msg in rx {
            msg.try_call(&o).unwrap();
        }
    });
    let p = MyIfaceProxy::<_, _>(|c| tx.send(c));
    dbg!(p.foo());
    dbg!(p.bar(4));
}
