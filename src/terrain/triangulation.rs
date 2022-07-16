use cgmath::Vector3;
use lepton::prelude::*;

struct Triangulation {
    data: [u8; 9],
    tri_count: usize,
}
impl Triangulation {
    const fn new0() -> Self {
        Self { 
            data: [0, 0, 0, 0, 0, 0, 0, 0, 0],
            tri_count: 0,
        }
    }
    const fn new1(data: [u8; 3]) -> Self {
        Self { 
            data: [data[0], data[1], data[2], 0, 0, 0, 0, 0, 0],
            tri_count: 1,
        }
    }
    const fn new2(data: [u8; 6]) -> Self {
        Self { 
            data: [data[0], data[1], data[2], data[3], data[4], data[5], 0, 0, 0],
            tri_count: 2,
        }
    }
    const fn new3(data: [u8; 9]) -> Self {
        Self { 
            data,
            tri_count: 3,
        }
    }
}

const TRIANGULATIONS: [Triangulation; 64] = [
    Triangulation::new0(),
    Triangulation::new1([2, 1, 3]),
    Triangulation::new1([0, 2, 4]),
    Triangulation::new2([01, 2, 3, 0, 3, 4]),
    Triangulation::new1([1, 0, 5]),
    Triangulation::new2([2, 0, 3, 0, 5, 3]),
    Triangulation::new2([2, 4, 1, 1, 4, 5]),
    Triangulation::new1([3, 4, 5]),

    Triangulation::new1([3, 0, 7]),
    Triangulation::new2([2, 1, 8, 8, 1, 7]),
    Triangulation::new2([3, 8, 7, 2, 0, 4]),
    Triangulation::new3([1, 7, 8, 8, 0, 1, 8, 4, 0]),
    Triangulation::new2([3, 7, 8, 1, 5, 0]),
    Triangulation::new3([5, 7, 8, 8, 0, 5, 8, 2, 0]),
    Triangulation::new3([1, 2, 3, 5, 7, 8, 8, 4, 5]),
    Triangulation::new2([5, 7, 8, 8, 4, 5]),

    Triangulation::new1([4, 8, 6]),
    Triangulation::new2([4, 8, 6, 2, 1, 3]),
    Triangulation::new2([0, 2, 8, 0, 8, 6]),
    Triangulation::new3([1, 3, 8, 8, 0, 1, 8, 6, 0]),
    Triangulation::new2([1, 0, 5, 4, 8, 6]),
    Triangulation::new3([4, 8, 6, 0, 2, 3, 3, 5, 0]),
    Triangulation::new3([5, 8, 6, 5, 1, 8, 1, 2, 8]),
    Triangulation::new2([5, 3, 8, 8, 6, 5]),

    Triangulation::new2([7, 3, 4, 4, 6, 7]),
    Triangulation::new3([6, 7, 1, 1, 4, 6, 1, 2, 4]),
    Triangulation::new3([7, 3, 6, 6, 3, 0, 0, 3, 2]),
    Triangulation::new2([7, 6, 1, 1, 6, 0]),
    Triangulation::new3([7, 6, 5, 1, 0, 3, 0, 4, 3]),
    Triangulation::new2([5, 7, 6, 2, 0, 4]),
    Triangulation::new2([5, 7, 6, 1, 2, 3]),
    Triangulation::new1([5, 7, 6]),

    Triangulation::new1([5, 6, 7]),
    Triangulation::new2([5, 6, 7, 2, 1, 3]),
    Triangulation::new2([5, 6, 7, 2, 1, 3]),
    Triangulation::new3([5, 6, 7, 1, 3, 4, 0, 1, 4]),
    Triangulation::new2([1, 0, 7, 0, 6, 7]),
    Triangulation::new3([3, 6, 7, 3, 2, 6, 2, 0, 6]),
    Triangulation::new3([7, 1, 6, 1, 2, 6, 2, 4, 6]),
    Triangulation::new2([3, 4, 7, 4, 6, 7]),

    Triangulation::new2([8, 3 ,5, 5, 6, 8]),
    Triangulation::new3([8, 5, 6, 8, 1, 5, 8, 2, 1]),
    Triangulation::new3([8, 3, 5, 5, 6, 8, 0, 2, 4]),
    Triangulation::new2([1, 0, 5, 4, 6, 8]),
    Triangulation::new3([3, 1, 8, 1, 0, 8, 0, 6, 8]),
    Triangulation::new2([2, 0, 8, 8, 0, 6]),
    Triangulation::new2([1, 2, 3, 4, 6, 8]),
    Triangulation::new1([4, 6, 8]),

    Triangulation::new2([7, 5, 8, 4, 8, 5]),
    Triangulation::new3([7, 5, 8, 4, 8, 5, 1, 3, 2]),
    Triangulation::new3([7, 5, 8, 5, 0, 8, 0, 2, 8]),
    Triangulation::new2([3, 7, 8, 1, 0, 5]),
    Triangulation::new3([1, 8, 7, 1, 0, 8, 0, 4, 8]),
    Triangulation::new2([3, 8, 7, 2, 0, 4]),
    Triangulation::new2([7, 1, 2, 2, 8, 7]),
    Triangulation::new1([3, 8, 7]),
    
    Triangulation::new1([4, 3, 5]),
    Triangulation::new2([2, 1, 5, 5, 4, 2]),
    Triangulation::new2([2, 3, 5, 5, 0, 2]),
    Triangulation::new1([0, 1, 5]),
    Triangulation::new2([0, 4, 3, 3, 1, 0]),
    Triangulation::new1([2, 0, 4]),
    Triangulation::new1([1, 2, 3]),
    Triangulation::new0()
];

pub fn assess_prism(corner_poses: [Vector3<f64>; 6], corner_vals: [f64; 6], vertices: &mut Vec<vertex::VertexLP>, indices: &mut Vec<u32>) {
    
}