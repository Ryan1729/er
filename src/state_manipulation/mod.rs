use common::*;

use rand::{Rng, SeedableRng, StdRng};

//NOTE(Ryan1729): debug_assertions only appears to work correctly when the
//crate is not a dylib. Assuming you make this crate *not* a dylib on release,
//these configs should work
#[cfg(debug_assertions)]
pub fn new_state(size: Size) -> State {
    println!("debug on. {:?}", size);

    let seed: &[_] = &[42];
    let rng: StdRng = SeedableRng::from_seed(seed);

    make_state(rng)
}

#[cfg(not(debug_assertions))]
pub fn new_state(size: Size) -> State {
    use std::time;
    let timestamp = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .map(|dur| dur.as_secs())
        .unwrap_or(42);

    println!("{}", timestamp);
    let seed: &[_] = &[timestamp as usize];
    let rng: StdRng = SeedableRng::from_seed(seed);

    make_state(rng)
}

fn make_state(mut rng: StdRng) -> State {
    rng.gen::<bool>();

    State {
        rng,
        ui_context: UIContext::new(),
    }
}

//returns true if quit requested
pub fn update_and_render(platform: &Platform, state: &mut State, events: &mut Vec<Event>) -> bool {
    let mut left_mouse_pressed = false;
    let mut left_mouse_released = false;

    for event in events {
        cross_mode_event_handling(platform, state, event);

        match *event {
            Event::KeyPressed {
                key: KeyCode::MouseLeft,
                ctrl: _,
                shift: _,
            } => {
                left_mouse_pressed = true;
            }
            Event::KeyReleased {
                key: KeyCode::MouseLeft,
                ctrl: _,
                shift: _,
            } => {
                left_mouse_released = true;
            }
            Event::Close
            | Event::KeyPressed {
                key: KeyCode::Escape,
                ctrl: _,
                shift: _,
            } => return true,
            _ => (),
        }
    }

    state.ui_context.frame_init();

    let button_spec = ButtonSpec {
        x: 0,
        y: 0,
        w: 11,
        h: 3,
        text: "Button".to_string(),
        id: 1,
    };

    if do_button(
        platform,
        &mut state.ui_context,
        &button_spec,
        left_mouse_pressed,
        left_mouse_released,
    ) {
        println!("Button pushed!");
    }

    //Demo:
    //1. Run `cargo run` in the folder containing the `state_manipulation` folder
    //   Leave the windoe open.
    //2. Change this string and save the file.
    //3. Run `cargo build` in the `state_manipulation` folder.
    //4. See that the string has changed in the running  program!
    (platform.print_xy)(34, 14, "Hello World!");

    false
}

fn cross_mode_event_handling(platform: &Platform, state: &mut State, event: &Event) {
    match *event {
        Event::KeyPressed {
            key: KeyCode::R,
            ctrl: true,
            shift: _,
        } => {
            println!("reset");
            *state = new_state((platform.size)());
        }
        _ => (),
    }
}

pub struct ButtonSpec {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub text: String,
    pub id: i32,
}

//calling this once will swallow multiple clicks on the button. We could either
//pass in and return the number of clicks to fix that, or this could simply be
//called multiple times per frame (once for each click).
fn do_button(
    platform: &Platform,
    context: &mut UIContext,
    spec: &ButtonSpec,
    left_mouse_pressed: bool,
    left_mouse_released: bool,
) -> bool {
    let mut result = false;

    let mouse_pos = (platform.mouse_position)();
    let inside = inside_rect(mouse_pos, spec.x, spec.y, spec.w, spec.h);
    let id = spec.id;

    if context.active == id {
        if left_mouse_released {
            result = context.hot == id && inside;

            context.set_not_active();
        }
    } else if context.hot == id {
        if left_mouse_pressed {
            context.set_active(id);
        }
    }

    if inside {
        context.set_next_hot(id);
    }

    if context.active == id && (platform.key_pressed)(KeyCode::MouseLeft) {
        draw_rect_with(
            platform,
            spec.x,
            spec.y,
            spec.w,
            spec.h,
            ["╔", "═", "╕", "║", "│", "╙", "─", "┘"],
        );
    } else if context.hot == id {
        draw_rect_with(
            platform,
            spec.x,
            spec.y,
            spec.w,
            spec.h,
            ["┌", "─", "╖", "│", "║", "╘", "═", "╝"],
        );
    } else {
        draw_rect(platform, spec.x, spec.y, spec.w, spec.h);
    }

    print_centered_line(platform, spec.x, spec.y, spec.w, spec.h, &spec.text);

    return result;
}

pub fn inside_rect(point: Point, x: i32, y: i32, w: i32, h: i32) -> bool {
    x <= point.x && y <= point.y && point.x < x + w && point.y < y + h
}

fn print_centered_line(platform: &Platform, x: i32, y: i32, w: i32, h: i32, text: &str) {
    let x_ = {
        let rect_middle = x + (w / 2);

        rect_middle - (text.chars().count() as f32 / 2.0) as i32
    };

    let y_ = y + (h / 2);

    (platform.print_xy)(x_, y_, &text);
}

fn draw_rect(platform: &Platform, x: i32, y: i32, w: i32, h: i32) {
    draw_rect_with(
        platform,
        x,
        y,
        w,
        h,
        ["┌", "─", "┐", "│", "│", "└", "─", "┘"],
    );
}

fn draw_rect_with(platform: &Platform, x: i32, y: i32, w: i32, h: i32, edges: [&str; 8]) {
    (platform.clear)(Some(Rect::from_values(x, y, w, h)));

    let right = x + w - 1;
    let bottom = y + h - 1;
    // top
    (platform.print_xy)(x, y, edges[0]);
    for i in (x + 1)..right {
        (platform.print_xy)(i, y, edges[1]);
    }
    (platform.print_xy)(right, y, edges[2]);

    // sides
    for i in (y + 1)..bottom {
        (platform.print_xy)(x, i, edges[3]);
        (platform.print_xy)(right, i, edges[4]);
    }

    //bottom
    (platform.print_xy)(x, bottom, edges[5]);
    for i in (x + 1)..right {
        (platform.print_xy)(i, bottom, edges[6]);
    }
    (platform.print_xy)(right, bottom, edges[7]);
}
