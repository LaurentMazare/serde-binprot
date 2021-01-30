mod de;
mod error;
mod ser;
const CODE_NEG_INT8: u8 = 0xff;
const CODE_INT16: u8 = 0xfe;
const CODE_INT32: u8 = 0xfd;
const CODE_INT64: u8 = 0xfc;

pub use crate::de::{from_reader, from_slice, from_str, Deserializer};
pub use crate::error::{Error, Result};
pub use crate::ser::{to_vec, to_writer, Serializer};

#[cfg(test)]
mod tests {
    use serde_derive::Deserialize;
    use serde_derive::Serialize;

    #[test]
    fn test_roundtrip() {
        #[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Copy)]
        struct Foo {
            foo_i32: i32,
            foo_i64: i64,
            foo_bool: bool,
        }

        #[derive(Deserialize, Serialize, PartialEq, Debug)]
        struct Bar {
            bar: (i32, Option<i32>),
            bar_str: String,
            bar_strs: Vec<(String, f64)>,
            bar_foo: Foo,
        }

        #[derive(Deserialize, Serialize, PartialEq, Debug)]
        enum FooBar {
            Foo(Foo),
            Bar(Bar),
            Inode(Vec<FooBar>),
        }

        let foo = Foo {
            foo_i32: -42,
            foo_i64: 1337133713371337,
            foo_bool: false,
        };
        let ser = crate::to_vec(&foo).unwrap();
        let de_foo: Foo = crate::from_slice(&ser).unwrap();
        assert_eq!(&foo, &de_foo);

        let bar = Bar {
            bar: (-42, None),
            bar_str: String::from("barbar"),
            bar_strs: vec![
                (String::from("pi"), 3.14159265),
                (String::from("e"), 2.718281828),
            ],
            bar_foo: foo,
        };
        let ser = crate::to_vec(&bar).unwrap();
        let de_bar: Bar = crate::from_slice(&ser).unwrap();
        assert_eq!(&bar, &de_bar);

        let foobar = FooBar::Foo(foo);
        let ser = crate::to_vec(&foobar).unwrap();
        let de_foobar: FooBar = crate::from_slice(&ser).unwrap();
        assert_eq!(foobar, de_foobar);

        let foobar = FooBar::Inode(vec![FooBar::Foo(foo), FooBar::Foo(foo)]);
        let foobar = FooBar::Inode(vec![foobar, FooBar::Foo(foo)]);
        let ser = crate::to_vec(&foobar).unwrap();
        let de_foobar: FooBar = crate::from_slice(&ser).unwrap();
        assert_eq!(foobar, de_foobar)
    }
}
