use serde::de::{Deserializer, SeqAccess};
use serde::ser::Serializer;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

pub trait KeyValue<'a>: serde::Serialize + serde::Deserialize<'a> {
    type Key: Eq + Hash;

    fn key(&self) -> Self::Key;
}

pub fn serialize<'a, S: Serializer, K: KeyValue<'a>>(
    map: &HashMap<K::Key, K>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(map.values())
}

pub fn deserialize<'de, D: Deserializer<'de>, K: KeyValue<'de>>(
    deserializer: D,
) -> Result<HashMap<K::Key, K>, D::Error> {
    struct Visitor<K>(PhantomData<K>);

    impl<'de, K: KeyValue<'de>> serde::de::Visitor<'de> for Visitor<K> {
        type Value = HashMap<K::Key, K>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence of items")
        }

        fn visit_seq<V: SeqAccess<'de>>(self, mut seq: V) -> Result<HashMap<K::Key, K>, V::Error> {
            let mut map = HashMap::with_capacity(seq.size_hint().unwrap_or(0));
            while let Some(item) = seq.next_element::<K>()? {
                map.insert(item.key(), item);
            }
            Ok(map)
        }
    }

    deserializer.deserialize_seq(Visitor::<K>(PhantomData))
}

#[cfg(test)]
#[test]
fn main() {
    use std::collections::HashMap;

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    pub struct Items {
        #[serde(with = "self")]
        pub items: HashMap<i64, Item>,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    pub struct Item {
        id: i64,
        info: String,
    }

    impl KeyValue<'_> for Item {
        type Key = i64;

        fn key(&self) -> Self::Key {
            self.id
        }
    }

    let j = r#"
        items:
            - id: 3
              info: 'three'
            - id: 2
              info: 'two'
        "#;

    let s = serde_yaml::from_str::<Items>(j).unwrap();
    println!("{:#?}", s);
    let s = serde_yaml::to_string(&s).unwrap();
    println!("{}", s);
}
