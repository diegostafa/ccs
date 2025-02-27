use std::collections::{HashMap, HashSet};

use ccs::lts::{Lts, Transition};
use ccs::process::Process;
use macroquad::miniquad::window::screen_size;
use macroquad::prelude::*;
use macroquad::rand::rand;

const ZOOM_MIN: f32 = 1.0 / 5000.0;
const ZOOM_MAX: f32 = 1.0;
const ZOOM_FACTOR: f32 = 1.05;
const PAN_STEP: f32 = 0.01;

const NODE_RAD: f32 = 100.0;
const NODE_COL: Color = BLACK;

const EDGE_SIZE: f32 = 5.0;
const EDGE_BG_COL: Color = BLACK;
const EDGE_FG_COL: Color = LIGHTGRAY;

const ARROW_LEN: f32 = 30.0;
const ARROW_COL: Color = WHITE;
const ARROW_RAD: f32 = std::f32::consts::PI / 4.0;

const TEXT_SIZE: f32 = 50.0;
const TEXT_COL: Color = WHITE;

pub async fn render_lts(lts: &Lts) {
    let mut cam = {
        let (w, h) = screen_size();
        Camera2D {
            zoom: (ZOOM_MIN, ZOOM_MIN * w / h).into(),
            ..Default::default()
        }
    };
    let mut nodes: HashMap<_, _> = lts.nodes().into_iter().map(|n| (n, (0.0, 0.0))).collect();
    circle_layout(&mut nodes, &cam);

    loop {
        match get_last_key_pressed() {
            Some(KeyCode::Escape) | Some(KeyCode::Q) => break,
            Some(KeyCode::R) => random_layout(&mut nodes, &cam),
            Some(KeyCode::C) => circle_layout(&mut nodes, &cam),
            _ => (),
        }

        update_camera(&mut cam);
        update_nodes(&mut nodes, &mut cam);
        set_camera(&cam);
        clear_background(DARKGRAY);
        render_nodes(lts, &nodes);
        next_frame().await
    }
}

fn update_camera(cam: &mut Camera2D) {
    let keys = get_keys_down();
    if keys.contains(&KeyCode::Right) {
        cam.offset.x -= PAN_STEP
    }
    if keys.contains(&KeyCode::Left) {
        cam.offset.x += PAN_STEP
    }
    if keys.contains(&KeyCode::Down) {
        cam.offset.y += PAN_STEP
    }
    if keys.contains(&KeyCode::Up) {
        cam.offset.y -= PAN_STEP
    }
    if is_mouse_button_down(MouseButton::Left) {
        let delta = mouse_delta_position();
        if delta.x != 0.0 || delta.y != 0.0 {
            cam.offset.x -= delta.x;
            cam.offset.y += delta.y;
        }
    }
    let mut v_scroll = mouse_wheel().1;
    if let Some('+') = get_char_pressed() {
        v_scroll = 1.0;
    }
    if keys.contains(&KeyCode::Minus) {
        v_scroll = -1.0;
    }
    if v_scroll != 0.0 {
        cam.zoom.x = if v_scroll > 0.0 {
            cam.zoom.x * ZOOM_FACTOR
        } else {
            cam.zoom.x / ZOOM_FACTOR
        };
        cam.zoom.x = cam.zoom.x.clamp(ZOOM_MIN, ZOOM_MAX);
    }

    let (w, h) = screen_size();
    cam.zoom.y = cam.zoom.x * w / h;
}
fn update_nodes(nodes: &mut HashMap<&Process, (f32, f32)>, cam: &mut Camera2D) {
    if is_mouse_button_down(MouseButton::Right) {
        let mpos = cam.screen_to_world(mouse_position().into());
        for (_, (x, y)) in nodes.iter_mut() {
            if Circle::new(*x, *y, NODE_RAD).contains(&mpos) {
                *x = mpos.x;
                *y = mpos.y;
                return;
            }
        }
    }
}

fn render_nodes(lts: &Lts, nodes: &HashMap<&Process, (f32, f32)>) {
    let is_bidir = |(p1, _, q1): &Transition| {
        lts.transitions()
            .iter()
            .any(|(p2, _, q2)| p1 == q2 && q1 == p2)
    };

    for (p, _, q) in lts.transitions() {
        let (px, py) = *nodes.get(&p).unwrap();
        let (qx, qy) = *nodes.get(&q).unwrap();
        draw_line(px, py, qx, qy, EDGE_SIZE, EDGE_BG_COL);
    }

    let mut drawn = HashSet::new();
    for ((p, ch, q), is_bidir) in lts.transitions().iter().map(|t| (t, is_bidir(t))) {
        let (px, py) = *nodes.get(&p).unwrap();

        if !drawn.contains(&p) {
            draw_circle(px, py, NODE_RAD, NODE_COL);
            draw_text(&p.to_string(), px, py - NODE_RAD, TEXT_SIZE, TEXT_COL);
            drawn.insert(p);
        }

        let (mut qx, mut qy) = *nodes.get(&q).unwrap();
        if !drawn.contains(&q) {
            draw_circle(qx, qy, NODE_RAD, NODE_COL);
            draw_text(&q.to_string(), qx, qy - NODE_RAD, TEXT_SIZE, TEXT_COL);
            drawn.insert(q);
        }

        if Circle::new(px, py, NODE_RAD).overlaps(&Circle::new(qx, qy, NODE_RAD)) {
            continue;
        }

        if is_bidir {
            qx = (px + qx) / 2.0;
            qy = (py + qy) / 2.0;
        }

        let (dx, dy) = (qx - px, qy - py);
        let dist = (dx * dx + dy * dy).sqrt();
        let (fi, mx, my) = (dy.atan2(dx), dx / dist, dy / dist);
        let (ex1, ey1) = (px + mx * NODE_RAD, py + my * NODE_RAD);
        let (ex2, ey2) = (qx - mx * NODE_RAD, qy - my * NODE_RAD);
        draw_line(ex1, ey1, ex2, ey2, EDGE_SIZE, EDGE_FG_COL);

        draw_text_ex(
            &ch.to_string(),
            (ex1 + ex2) / 2.0,
            (ey1 + ey2) / 2.0,
            TextParams {
                font: None,
                font_size: TEXT_SIZE as u16,
                font_scale: 1.0,
                font_scale_aspect: 1.0,
                rotation: fi + std::f32::consts::PI / 2.0,
                color: TEXT_COL,
            },
        );

        let ax1 = ex2 - ARROW_LEN * (fi + ARROW_RAD).cos();
        let ay1 = ey2 - ARROW_LEN * (fi + ARROW_RAD).sin();
        let ax2 = ex2 - ARROW_LEN * (fi - ARROW_RAD).cos();
        let ay2 = ey2 - ARROW_LEN * (fi - ARROW_RAD).sin();
        draw_triangle(
            (ex2 + mx * ARROW_LEN, ey2 + my * ARROW_LEN).into(),
            (ax1, ay1).into(),
            (ax2, ay2).into(),
            ARROW_COL,
        );
    }
}

fn random_layout(nodes: &mut HashMap<&Process, (f32, f32)>, cam: &Camera2D) {
    let (ww, wh) = screen_to_world_area(cam);

    for (_, (x, y)) in nodes.iter_mut() {
        *x = (rand() % ww as u32) as f32;
        *y = (rand() % wh as u32) as f32;
    }
}
fn circle_layout(nodes: &mut HashMap<&Process, (f32, f32)>, cam: &Camera2D) {
    let (w, h) = screen_size();
    let (ww, wh) = screen_to_world_area(cam);
    let c = cam.screen_to_world((w / 2.0, h / 2.0).into());
    let r = (ww.min(wh) / 2.0) - NODE_RAD * 10.0;
    let fi = (std::f32::consts::PI * 2.0) / nodes.len() as f32;

    for (i, (_, (x, y))) in nodes.iter_mut().enumerate() {
        let angle = fi * i as f32;
        *x = c.x + r * angle.cos();
        *y = c.y + r * angle.sin();
    }
}
fn screen_to_world_area(cam: &Camera2D) -> (f32, f32) {
    let (w, h) = screen_size();
    let tl = cam.screen_to_world((0.0, 0.0).into());
    let tr = cam.screen_to_world((w, 0.0).into());
    let bl = cam.screen_to_world((0.0, h).into());
    let ww = (tr.x - tl.x).abs();
    let wh = (bl.y - tl.y).abs();
    (ww, wh)
}
