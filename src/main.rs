mod gl;
mod shaders;

use dioxus::html::geometry::WheelDelta;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use std::time::Duration;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

use crate::gl::Renderer;

const BASE_CELL_W: f64 = 10.0;
const BASE_CELL_H: f64 = 16.0;
const TRAIL_MAX: usize = 16;
const HOLD_SPAWN_INTERVAL: f64 = 0.35;

fn viewport_size() -> (f64, f64) {
    web_sys::window()
        .map(|w| {
            let width = w
                .inner_width()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(1280.0);
            let height = w
                .inner_height()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(720.0);
            (width, height)
        })
        .unwrap_or((1280.0, 720.0))
}

fn dims_from_viewport(cell_w: f64, cell_h: f64) -> (usize, usize) {
    let (w, h) = viewport_size();
    let cols = ((w / cell_w) as usize).max(1);
    let rows = ((h / cell_h) as usize).max(1);
    (cols, rows)
}

fn find_canvas() -> Option<HtmlCanvasElement> {
    web_sys::window()?
        .document()?
        .get_element_by_id("wave-canvas")?
        .dyn_into::<HtmlCanvasElement>()
        .ok()
}

// Brand-mark SVGs, paths from each service's public icon set. `currentColor`
// lets the anchor's CSS color flow through.
const MASTODON_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="36" height="36" fill="currentColor" aria-hidden="true"><path d="M23.193 7.88c0-5.207-3.411-6.733-3.411-6.733C18.062.357 15.108.015 12.041 0h-.076c-3.068.015-6.02.357-7.74 1.147 0 0-3.412 1.526-3.412 6.732 0 1.193-.023 2.619.015 4.13.124 5.091.934 10.109 5.641 11.355 2.17.574 4.034.695 5.535.612 2.722-.15 4.25-.972 4.25-.972l-.09-1.975s-1.945.613-4.129.539c-2.165-.074-4.449-.233-4.799-2.891a5.499 5.499 0 0 1-.048-.745s2.125.519 4.817.642c1.646.075 3.19-.097 4.758-.283 3.007-.359 5.625-2.212 5.954-3.905.52-2.666.475-6.506.475-6.506zm-4.024 6.709h-2.497V8.469c0-1.29-.543-1.944-1.628-1.944-1.2 0-1.802.776-1.802 2.312v3.349h-2.483v-3.35c0-1.536-.602-2.312-1.802-2.312-1.085 0-1.628.655-1.628 1.944v6.12H4.832V8.284c0-1.289.328-2.313.987-3.07.68-.758 1.569-1.146 2.674-1.146 1.278 0 2.246.491 2.886 1.474L12 6.585l.621-1.043c.64-.983 1.608-1.474 2.886-1.474 1.104 0 1.994.388 2.674 1.146.658.757.986 1.781.986 3.07v6.305z"/></svg>"#;

const GITHUB_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="36" height="36" fill="currentColor" aria-hidden="true"><path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23a11.51 11.51 0 0 1 3-.405c1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/></svg>"#;

const LINKEDIN_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="36" height="36" fill="currentColor" aria-hidden="true"><path d="M19 0h-14c-2.761 0-5 2.239-5 5v14c0 2.761 2.239 5 5 5h14c2.762 0 5-2.239 5-5v-14c0-2.761-2.238-5-5-5zm-11 19h-3v-11h3v11zm-1.5-12.268c-.966 0-1.75-.79-1.75-1.764s.784-1.764 1.75-1.764 1.75.79 1.75 1.764-.783 1.764-1.75 1.764zm13.5 12.268h-3v-5.604c0-3.368-4-3.113-4 0v5.604h-3v-11h3v1.765c1.396-2.586 7-2.777 7 2.476v6.759z"/></svg>"#;

fn wheel_delta_y(d: WheelDelta) -> f64 {
    match d {
        WheelDelta::Pixels(p) => p.y,
        WheelDelta::Lines(p) => p.y * 20.0,
        WheelDelta::Pages(p) => p.y * 500.0,
    }
}

#[allow(non_snake_case)]
fn App() -> Element {
    let mut zoom: Signal<f64> = use_signal(|| 1.0);
    let mut dims: Signal<(usize, usize)> =
        use_signal(|| dims_from_viewport(BASE_CELL_W, BASE_CELL_H));
    let mut time: Signal<f64> = use_signal(|| 0.0);
    let mut mouse_grid: Signal<(f64, f64)> = use_signal(|| {
        let (c, r) = dims_from_viewport(BASE_CELL_W, BASE_CELL_H);
        (c as f64 * 0.5, r as f64 * 0.5)
    });
    // (x, y, birth, sign); sign = -1 for right-click void pulses.
    let mut click_pulses: Signal<Vec<(f64, f64, f64, f64)>> = use_signal(Vec::new);
    let mut mouse_strength: Signal<f64> = use_signal(|| 1.0);
    let mut last_mouse: Signal<Option<(f64, f64, f64)>> = use_signal(|| None);
    // Some(sign) while a button is pressed; drives click-and-hold spawns.
    let mut mouse_down: Signal<Option<f64>> = use_signal(|| None);

    // Wire up window resize → dims signal (respects current zoom).
    use_hook(|| {
        let Some(window) = web_sys::window() else {
            return;
        };
        let closure = Closure::<dyn FnMut()>::new(move || {
            let z = *zoom.peek();
            dims.set(dims_from_viewport(BASE_CELL_W * z, BASE_CELL_H * z));
        });
        window.set_onresize(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    });

    use_future(move || async move {
        let mut renderer: Option<Renderer> = None;
        let mut last_dims: (usize, usize) = (0, 0);
        let mut last_hold_spawn: f64 = -1000.0;
        let mut trail: Vec<(f32, f32, f32)> = Vec::with_capacity(TRAIL_MAX);

        loop {
            gloo_timers::future::sleep(Duration::from_millis(16)).await;

            let t = *time.peek() + 0.016;
            time.set(t);

            click_pulses.with_mut(|p| p.retain(|&(_, _, bt, _)| t - bt < 4.0));

            // Decay strength toward 1.0 (~6%/frame).
            let s = *mouse_strength.peek();
            mouse_strength.set(s * 0.94 + 0.06);

            // Click-and-hold: keep spawning pulses while a button is held.
            if let Some(sign) = *mouse_down.peek() {
                if t - last_hold_spawn >= HOLD_SPAWN_INTERVAL {
                    last_hold_spawn = t;
                    let mg = *mouse_grid.peek();
                    click_pulses.with_mut(|p| {
                        p.push((mg.0, mg.1, t, sign));
                        if p.len() > 10 {
                            p.remove(0);
                        }
                    });
                }
            }

            // Trail: push current mouse position if it moved or enough time passed,
            // then drop anything older than 0.8 s.
            let mg = *mouse_grid.peek();
            let need_push = match trail.last() {
                Some(&(lx, ly, lt)) => {
                    let d = ((mg.0 as f32 - lx).powi(2) + (mg.1 as f32 - ly).powi(2)).sqrt();
                    d > 0.4 || (t as f32 - lt) > 0.04
                }
                None => true,
            };
            if need_push {
                trail.push((mg.0 as f32, mg.1 as f32, t as f32));
                if trail.len() > TRAIL_MAX {
                    trail.remove(0);
                }
            }
            trail.retain(|&(_, _, bt)| (t as f32 - bt) < 0.8);

            if renderer.is_none() {
                let Some(canvas) = find_canvas() else {
                    continue;
                };
                renderer = Some(Renderer::new(&canvas, BASE_CELL_W, BASE_CELL_H));
            }
            let Some(ref r) = renderer else {
                continue;
            };

            let z = *zoom.peek();
            let cw = BASE_CELL_W * z;
            let ch = BASE_CELL_H * z;

            let (cols, rows) = *dims.peek();
            let canvas_w = cols as f64 * cw;
            let canvas_h = rows as f64 * ch;
            if (cols, rows) != last_dims {
                last_dims = (cols, rows);
                r.set_viewport(canvas_w as i32, canvas_h as i32);
            }

            let pulses_snap: Vec<(f32, f32, f32, f32)> = click_pulses
                .peek()
                .iter()
                .map(|&(x, y, bt, sign)| (x as f32, y as f32, bt as f32, sign as f32))
                .collect();

            r.draw(
                t as f32,
                canvas_w as f32,
                canvas_h as f32,
                cw as f32,
                ch as f32,
                (mg.0 as f32, mg.1 as f32),
                s as f32,
                &pulses_snap,
                &trail,
            );
        }
    });

    let (cols, rows) = dims();
    let z = zoom();
    let cw = BASE_CELL_W * z;
    let ch = BASE_CELL_H * z;
    let canvas_w = cols as f64 * cw;
    let canvas_h = rows as f64 * ch;

    rsx! {
        div {
            style: "
                background: #000;
                margin: 0;
                padding: 0;
                width: 100vw;
                height: 100vh;
                overflow: hidden;
                user-select: none;
                cursor: crosshair;
            ",
            canvas {
                id: "wave-canvas",
                width: "{canvas_w}",
                height: "{canvas_h}",
                style: "display: block;",
                oncontextmenu: move |evt| {
                    evt.prevent_default();
                },
                onwheel: move |evt| {
                    evt.prevent_default();
                    let dy = wheel_delta_y(evt.data().delta());
                    let current = *zoom.peek();
                    let new_zoom = (current * (1.0 - dy * 0.001)).clamp(0.3, 3.0);
                    zoom.set(new_zoom);
                    dims.set(dims_from_viewport(BASE_CELL_W * new_zoom, BASE_CELL_H * new_zoom));
                },
                onmousemove: move |evt| {
                    let c = evt.element_coordinates();
                    let z = *zoom.peek();
                    let cw = BASE_CELL_W * z;
                    let ch = BASE_CELL_H * z;
                    let pos = (c.x / cw, c.y / ch);
                    mouse_grid.set(pos);

                    let t = *time.peek();
                    if let Some((lx, ly, lt)) = *last_mouse.peek() {
                        let dx = pos.0 - lx;
                        let dy = pos.1 - ly;
                        let dt = (t - lt).max(0.001);
                        let speed = (dx * dx + dy * dy).sqrt() / dt;
                        let target = (1.0 + speed * 0.015).min(6.0);
                        let cur = *mouse_strength.peek();
                        mouse_strength.set(cur * 0.6 + target * 0.4);
                    }
                    last_mouse.set(Some((pos.0, pos.1, t)));
                },
                onmousedown: move |evt| {
                    let data = evt.data();
                    let is_right = matches!(data.trigger_button(), Some(MouseButton::Secondary));
                    let sign: f64 = if is_right { -1.0 } else { 1.0 };
                    let c = data.element_coordinates();
                    let z = *zoom.peek();
                    let cw = BASE_CELL_W * z;
                    let ch = BASE_CELL_H * z;
                    let pos = (c.x / cw, c.y / ch);
                    let t = *time.peek();
                    click_pulses.with_mut(|p| {
                        p.push((pos.0, pos.1, t, sign));
                        if p.len() > 10 { p.remove(0); }
                    });
                    mouse_down.set(Some(sign));
                },
                onmouseup: move |_| {
                    mouse_down.set(None);
                },
                onmouseleave: move |_| {
                    mouse_down.set(None);
                },
            }
            div {
                // Overlay is pointer-transparent so the canvas below remains
                // fully interactive; the link tags re-enable pointer events.
                style: "
                    position: fixed;
                    inset: 0;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    pointer-events: none;
                    z-index: 10;
                ",
                div {
                    style: "
                        text-align: center;
                        color: #fff;
                        font-family: 'Montserrat', sans-serif;
                        text-shadow: 0 0 12px rgba(0,0,0,0.9),
                                     0 0 28px rgba(0,0,0,0.75),
                                     0 2px 4px rgba(0,0,0,0.95);
                    ",
                    h1 {
                        style: "font-size: clamp(48px, 10vw, 112px); margin: 0; letter-spacing: 0.05em; font-weight: 700;",
                        "Skabber"
                    }
                    p {
                        style: "font-size: clamp(16px, 2.2vw, 24px); margin: 8px 0 32px; font-weight: 400; opacity: 0.95;",
                        "Hi There!"
                    }
                    div {
                        style: "display: flex; gap: 28px; justify-content: center;",
                        a {
                            class: "overlay-link",
                            href: "https://hachyderm.io/@Skabber",
                            target: "_blank",
                            rel: "me noopener noreferrer",
                            "aria-label": "Mastodon",
                            dangerous_inner_html: MASTODON_SVG,
                        }
                        a {
                            class: "overlay-link",
                            href: "https://github.com/skabber",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            "aria-label": "GitHub",
                            dangerous_inner_html: GITHUB_SVG,
                        }
                        a {
                            class: "overlay-link",
                            href: "https://lnkd.in/skabber",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            "aria-label": "LinkedIn",
                            dangerous_inner_html: LINKEDIN_SVG,
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    dioxus::LaunchBuilder::new()
        .with_cfg(dioxus::web::Config::default().rootname("main"))
        .launch(App);
}
