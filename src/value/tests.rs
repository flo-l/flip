use ::value::Value;
use ::string_interner::StringInterner;

#[test]
fn pair_format() {
    fn v(x: i64) -> Value { Value::new_integer(x) }
    let interner = &mut StringInterner::new();
    let empty = Value::empty_list();

    let a = Value::new_pair(v(4), empty.clone());
    let b = Value::new_pair(v(3), a.clone());
    let c = Value::new_pair(v(2), b.clone());
    let d = Value::new_pair(v(1), c.clone());

    assert_eq!(empty.to_string(interner), "()");
    assert_eq!(a.to_string(interner), "(4)");
    assert_eq!(b.to_string(interner), "(3 4)");
    assert_eq!(c.to_string(interner), "(2 3 4)");
    assert_eq!(d.to_string(interner), "(1 2 3 4)");

    let x = Value::new_pair(v(3), v(4));
    let y = Value::new_pair(v(2), x.clone());
    let z = Value::new_pair(v(1), y.clone());

    assert_eq!(x.to_string(interner), "(3 . 4)");
    assert_eq!(y.to_string(interner), "(2 (3 . 4))");
    assert_eq!(z.to_string(interner), "(1 2 (3 . 4))");

    let r = Value::new_pair(v(4), empty.clone());
    let s = Value::new_pair(v(2), v(3));
    let t = Value::new_pair(s.clone(), r.clone());
    let u = Value::new_pair(v(1), t.clone());
    let v = Value::new_pair(v(0), u.clone());

    assert_eq!(r.to_string(interner), "(4)");
    assert_eq!(s.to_string(interner), "(2 . 3)");
    assert_eq!(t.to_string(interner), "((2 . 3) 4)");
    assert_eq!(u.to_string(interner), "(1 (2 . 3) 4)");
    assert_eq!(v.to_string(interner), "(0 1 (2 . 3) 4)");
}
