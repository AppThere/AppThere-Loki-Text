use super::*;

const EPS: f64 = 1e-9;

fn eq(a: f64, b: f64) -> bool {
    (a - b).abs() < EPS
}

#[test]
fn test_identity_apply() {
    let t = Transform::identity();
    let (x, y) = t.apply(3.0, 4.0);
    assert!(eq(x, 3.0) && eq(y, 4.0));
}

#[test]
fn test_translate() {
    let t = Transform::translate(10.0, 20.0);
    let (x, y) = t.apply(0.0, 0.0);
    assert!(eq(x, 10.0) && eq(y, 20.0));
}

#[test]
fn test_scale() {
    let t = Transform::scale(2.0, 3.0);
    let (x, y) = t.apply(5.0, 4.0);
    assert!(eq(x, 10.0) && eq(y, 12.0));
}

#[test]
fn test_rotate_90() {
    let t = Transform::rotate(90.0);
    let (x, y) = t.apply(1.0, 0.0);
    assert!(eq(x, 0.0) && eq(y, 1.0), "got ({}, {})", x, y);
}

#[test]
fn test_multiply_associativity() {
    let a = Transform::translate(5.0, 0.0);
    let b = Transform::scale(2.0, 2.0);
    let c = Transform::rotate(45.0);
    let ab_c = a.multiply(&b).multiply(&c);
    let a_bc = a.multiply(&b.multiply(&c));
    assert!(eq(ab_c.a, a_bc.a) && eq(ab_c.e, a_bc.e));
}

#[test]
fn test_svg_matrix_roundtrip() {
    let t = Transform { a: 1.0, b: 0.5, c: -0.5, d: 1.0, e: 10.0, f: 20.0 };
    let s = t.to_svg_matrix();
    let back = Transform::from_svg_matrix(&s).unwrap();
    assert!(eq(t.a, back.a) && eq(t.e, back.e) && eq(t.f, back.f));
}

#[test]
fn test_is_identity() {
    assert!(Transform::identity().is_identity());
    assert!(!Transform::translate(1.0, 0.0).is_identity());
}
