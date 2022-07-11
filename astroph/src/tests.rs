use crate::prelude::*;
use std::path::Path;
const PERCENTILE: f32 = 0.9;

const WIDTH: usize = 500;
const HEIGHT: usize = 500;
const BRIGHTNESS: f32 = 0.5;

#[test]
fn get_top_map() {
    let g = Galaxy::default();
    let pixels = g.render(WIDTH, HEIGHT, Direction::Down, [0.0, 0.0, 25_000.0]);
    let mut brightnesses = pixels.concat().iter().map(|c| { f32::max((c.0 + c.1 + c.2) / 3.0, 0.0) }).collect::<Vec<_>>();
    brightnesses.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = brightnesses[(WIDTH as f32 * HEIGHT as f32 * PERCENTILE) as usize];

    let mut buffer = [0; WIDTH * HEIGHT * 3];
    let mut buffer_index = 0;
    for line in pixels {
        for p in line {
            buffer[buffer_index + 0] = (p.0 * 256.0 / median * BRIGHTNESS) as u8;
            buffer[buffer_index + 1] = (p.1 * 256.0 / median * BRIGHTNESS) as u8;
            buffer[buffer_index + 2] = (p.2 * 256.0 / median * BRIGHTNESS) as u8;
            buffer_index += 3;
        }
    }
    image::save_buffer(&Path::new("top.png"), &buffer, WIDTH as u32, HEIGHT as u32, image::ColorType::Rgb8).unwrap();
}
#[test]
fn get_side_map() {
    let g = Galaxy::default();
    let pixels = g.render(WIDTH, HEIGHT, Direction::Forward, [-35_000.0, 0.0, -5_000.0]);
    let mut brightnesses = pixels.concat().iter().map(|c| { f32::max((c.0 + c.1 + c.2) / 3.0, 0.0) }).collect::<Vec<_>>();
    brightnesses.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = brightnesses[(WIDTH as f32 * HEIGHT as f32 * PERCENTILE) as usize];

    let mut buffer = [0; WIDTH * HEIGHT * 3];
    let mut buffer_index = 0;
    for line in pixels {
        for p in line {
            buffer[buffer_index + 0] = (p.0 * 256.0 / median * BRIGHTNESS) as u8;
            buffer[buffer_index + 1] = (p.1 * 256.0 / median * BRIGHTNESS) as u8;
            buffer[buffer_index + 2] = (p.2 * 256.0 / median * BRIGHTNESS) as u8;
            buffer_index += 3;
        }
    }
    image::save_buffer(&Path::new("side.png"), &buffer, WIDTH as u32, HEIGHT as u32, image::ColorType::Rgb8).unwrap();
}

#[test]
fn skybox() {
    let g = Galaxy::default();
    let pos = [15_000.0, 15_000.0, 1000.0];
    let pixels = vec![
        g.render(WIDTH, HEIGHT, Direction::Up, pos),
        g.render(WIDTH, HEIGHT, Direction::Down, pos),
        g.render(WIDTH, HEIGHT, Direction::Forward, pos),
        g.render(WIDTH, HEIGHT, Direction::Backward, pos),
        g.render(WIDTH, HEIGHT, Direction::Left, pos),
        g.render(WIDTH, HEIGHT, Direction::Right, pos),
    ];
    let names = vec![
        "up.png",
        "down.png",
        "forward.png",
        "backward.png",
        "left.png",
        "right.png",
    ];
    let mut brightnesses = pixels.concat().concat().iter()
        .map(|c| { f32::max((c.0 + c.1 + c.2) / 3.0, 0.0) }).collect::<Vec<_>>();
    brightnesses.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = brightnesses[(6.0 * WIDTH as f32 * HEIGHT as f32 * PERCENTILE) as usize];

    for (pixels, name) in pixels.iter().zip(names) {
        let mut buffer = [0; WIDTH * HEIGHT * 3];
        let mut buffer_index = 0;
        for line in pixels {
            for p in line {
                buffer[buffer_index + 0] = (p.0 * 256.0 / median * BRIGHTNESS) as u8;
                buffer[buffer_index + 1] = (p.1 * 256.0 / median * BRIGHTNESS) as u8;
                buffer[buffer_index + 2] = (p.2 * 256.0 / median * BRIGHTNESS) as u8;
                buffer_index += 3;
            }
        }
        image::save_buffer(&Path::new(name), &buffer, WIDTH as u32, HEIGHT as u32, image::ColorType::Rgb8).unwrap();
    }
}