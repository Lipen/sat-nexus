use quickcheck::{Arbitrary, Gen};
use quickcheck_macros::quickcheck;

use super::*;

#[derive(Debug, PartialEq, Clone, Copy)]
struct Value(i32);

impl Arbitrary for Value {
    fn arbitrary(g: &mut Gen) -> Self {
        Value(i32::arbitrary(g))
    }
}

#[test]
fn test_it_works() {
    let mut context = Context::new();

    let forty_two = Value(42);
    let ten = Value(10);
    let eleven = Value(11);
    let five = Value(5);

    context.insert(forty_two);
    context.insert_named("ten", ten);
    context.insert_named("eleven".to_string(), eleven);
    context.insert(five);

    let extracted = context.get::<Value>().unwrap();
    assert_eq!(extracted, &five);
    let extracted = context.extract::<Value>();
    assert_eq!(extracted, &five);
    let extracted = context.extract_named::<Value, _>("ten".to_string());
    assert_eq!(extracted, &ten);
    let extracted = context.extract_named::<Value, _>("eleven");
    assert_eq!(extracted, &eleven);
}

#[quickcheck]
fn insert_extract(value: Value) -> bool {
    let mut context = Context::new();
    context.insert(value);
    let extracted = *context.extract::<Value>();
    extracted == value
}

#[quickcheck]
fn multi_insert_extract(values: Vec<Value>) -> bool {
    if values.is_empty() {
        return true;
    }
    let mut context = Context::new();
    let last = *values.last().unwrap();
    for value in values.into_iter() {
        context.insert(value);
    }
    let extracted = *context.extract::<Value>();
    extracted == last
}

#[quickcheck]
fn insert_named_extract(value: Value, name: String) -> bool {
    let mut context = Context::new();
    context.insert_named(name.clone(), value);
    let extracted = *context.extract_named::<Value, _>(name);
    extracted == value
}
