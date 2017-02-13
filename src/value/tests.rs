use ::value::Value;
use ::string_interner::StringInterner;

#[test]
fn list_format() {
    fn v(x: i64) -> Value { Value::new_integer(x) }
    let interner = &mut StringInterner::new();

    let empty = Value::empty_list();
    let a = Value::new_list(&[v(4)]);
    let b = Value::new_list(&[v(3), v(4)]);
    let c = Value::new_list(&[v(2), v(3), v(4)]);
    let d = Value::new_list(&[v(1), v(2), v(3), v(4)]);

    assert_eq!(empty.to_string(interner), "()");
    assert_eq!(a.to_string(interner), "(4)");
    assert_eq!(b.to_string(interner), "(3 4)");
    assert_eq!(c.to_string(interner), "(2 3 4)");
    assert_eq!(d.to_string(interner), "(1 2 3 4)");
}
