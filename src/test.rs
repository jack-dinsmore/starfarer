use cgmath::{Vector3, InnerSpace};

#[test]
fn cross_test() {
    let res = Vector3::new(1.0, 0.0, 0.0).cross(Vector3::new(0.0, 1.0, 0.0));
    println!("{:?}", res);
    assert_eq!(res.dot(Vector3::new(0.0, 0.0, 1.0)) > 0.0, true);
}