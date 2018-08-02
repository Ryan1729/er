use common::*;

use rand::{Rng, SeedableRng, StdRng};
use std::borrow::Cow;
use std::env;
use std::path::{Path, PathBuf};

//NOTE(Ryan1729): debug_assertions only appears to work correctly when the
//crate is not a dylib. Assuming you make this crate *not* a dylib on release,
//these configs should work
#[cfg(debug_assertions)]
pub fn new_state(size: Size) -> State {
    println!("debug on. {:?}", size);

    let seed: &[_] = &[42];
    let rng: StdRng = SeedableRng::from_seed(seed);

    State::new(rng, size)
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

    State::new(rng, size)
}

//returns true if quit requested
pub fn update_and_render(platform: &Platform, state: &mut State, events: &mut Vec<Event>) -> bool {
    state.left_mouse_pressed = false;
    state.left_mouse_released = false;

    for event in events {
        cross_mode_event_handling(platform, state, event);
        typing_events(platform, state, event);

        match *event {
            Event::KeyPressed {
                key: KeyCode::MouseLeft,
                ctrl: _,
                shift: _,
            } => {
                state.left_mouse_pressed = true;
            }
            Event::KeyReleased {
                key: KeyCode::MouseLeft,
                ctrl: _,
                shift: _,
            } => {
                state.left_mouse_released = true;
            }
            Event::Close
            | Event::KeyPressed {
                key: KeyCode::Escape,
                ctrl: _,
                shift: _,
            } => return true,
            Event::KeyReleased {
                key: KeyCode::Enter,
                ctrl: _,
                shift: _,
            } => {
                execute_command(state);
            }
            _ => (),
        }
    }

    state.ui_context.frame_init();

    (platform.print_xy)(0, 0, &state.prompt);

    let mut x = state.prompt.len() as i32;
    let mut y = 0;

    (platform.print_xy)(x, y, &state.command);

    y += 1;

    (platform.print_xy)(0, y, &state.output);

    false
}

fn execute_command(state: &mut State) {
    let space_at = state.command.find(" ").unwrap_or(state.command.len());
    let (raw_exe_name, args) = state.command.split_at(space_at);

    use std::fmt::Write;
    if let Some(exe_name) = find_exe_name(Path::new(raw_exe_name)) {
        use std::ffi;

        let path = env::var_os("PATH").unwrap_or(ffi::OsString::new());

        write!(&mut state.output, "{:?} {}\n", exe_name.as_os_str(), args);

        use std::process::Command;
        let output = Command::new(exe_name)
            .arg(args)
            .env("PATH", path)
            .output()
            .expect("failed to execute process");

        println!("status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

        write!(
            &mut state.output,
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
    } else {
        write!(&mut state.output, "Could not find \"{}\"!", raw_exe_name);
    }
}

// derived from https://stackoverflow.com/a/37499032/4496839
fn find_exe_name<P>(raw_exe_name: P) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    let exe_name = enhance_exe_name(raw_exe_name.as_ref());

    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(&exe_name);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    })
}

#[cfg(not(target_os = "windows"))]
fn enhance_exe_name(exe_name: &Path) -> Cow<Path> {
    exe_name.into()
}

#[cfg(target_os = "windows")]
fn enhance_exe_name(exe_name: &Path) -> Cow<Path> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let raw_input: Vec<_> = exe_name.as_os_str().encode_wide().collect();
    let raw_extension: Vec<_> = OsStr::new(".exe").encode_wide().collect();

    if raw_input.ends_with(&raw_extension) {
        exe_name.into()
    } else {
        let mut with_exe = exe_name.as_os_str().to_owned();
        with_exe.push(".exe");
        PathBuf::from(with_exe).into()
    }
}

/// Example:
///```
///    state.ui_context.frame_init();
///
///    let button_spec = ButtonSpec {
///        x: 0,
///        y: 0,
///        w: 11,
///        h: 3,
///        text: "Button".to_string(),
///        id: 1,
///    };
///
///    if do_button(
///        platform,
///        &mut state.ui_context,
///        &button_spec,
///        left_mouse_pressed,
///        left_mouse_released,
///    ) {
///        println!("Button pushed!");
///    }
///```
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

fn typing_events(platform: &Platform, state: &mut State, event: &Event) {
    match *event {
        Event::KeyPressed {
            key: KeyCode::Backspace,
            ctrl: _,
            shift: _,
        } => {
            state.command.pop();
        }
        Event::KeyPressed {
            key: KeyCode::Space,
            ctrl: _,
            shift: _,
        } => {
            state.command.push(' ');
        }
        Event::KeyPressed {
            key: KeyCode::A,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('A');
        }
        Event::KeyPressed {
            key: KeyCode::B,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('B');
        }
        Event::KeyPressed {
            key: KeyCode::C,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('C');
        }
        Event::KeyPressed {
            key: KeyCode::D,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('D');
        }
        Event::KeyPressed {
            key: KeyCode::E,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('E');
        }
        Event::KeyPressed {
            key: KeyCode::F,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('F');
        }
        Event::KeyPressed {
            key: KeyCode::G,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('G');
        }
        Event::KeyPressed {
            key: KeyCode::H,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('H');
        }
        Event::KeyPressed {
            key: KeyCode::I,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('I');
        }
        Event::KeyPressed {
            key: KeyCode::J,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('J');
        }
        Event::KeyPressed {
            key: KeyCode::K,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('K');
        }
        Event::KeyPressed {
            key: KeyCode::L,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('L');
        }
        Event::KeyPressed {
            key: KeyCode::M,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('M');
        }
        Event::KeyPressed {
            key: KeyCode::N,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('N');
        }
        Event::KeyPressed {
            key: KeyCode::O,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('O');
        }
        Event::KeyPressed {
            key: KeyCode::P,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('P');
        }
        Event::KeyPressed {
            key: KeyCode::Q,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('Q');
        }
        Event::KeyPressed {
            key: KeyCode::R,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('R');
        }
        Event::KeyPressed {
            key: KeyCode::S,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('S');
        }
        Event::KeyPressed {
            key: KeyCode::T,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('T');
        }
        Event::KeyPressed {
            key: KeyCode::U,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('U');
        }
        Event::KeyPressed {
            key: KeyCode::V,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('V');
        }
        Event::KeyPressed {
            key: KeyCode::W,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('W');
        }
        Event::KeyPressed {
            key: KeyCode::X,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('X');
        }
        Event::KeyPressed {
            key: KeyCode::Y,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('Y');
        }
        Event::KeyPressed {
            key: KeyCode::Z,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('Z');
        }
        Event::KeyPressed {
            key: KeyCode::Row1,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('!');
        }
        Event::KeyPressed {
            key: KeyCode::Row2,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('@');
        }
        Event::KeyPressed {
            key: KeyCode::Row3,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('#');
        }
        Event::KeyPressed {
            key: KeyCode::Row4,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('$');
        }
        Event::KeyPressed {
            key: KeyCode::Row5,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('%');
        }
        Event::KeyPressed {
            key: KeyCode::Row6,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('^');
        }
        Event::KeyPressed {
            key: KeyCode::Row7,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('&');
        }
        Event::KeyPressed {
            key: KeyCode::Row8,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('*');
        }
        Event::KeyPressed {
            key: KeyCode::Row9,
            ctrl: _,
            shift: true,
        } => {
            state.command.push('(');
        }
        Event::KeyPressed {
            key: KeyCode::Row0,
            ctrl: _,
            shift: true,
        } => {
            state.command.push(')');
        }
        Event::KeyPressed {
            key: KeyCode::A,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('a');
        }
        Event::KeyPressed {
            key: KeyCode::B,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('b');
        }
        Event::KeyPressed {
            key: KeyCode::C,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('c');
        }
        Event::KeyPressed {
            key: KeyCode::D,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('d');
        }
        Event::KeyPressed {
            key: KeyCode::E,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('e');
        }
        Event::KeyPressed {
            key: KeyCode::F,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('f');
        }
        Event::KeyPressed {
            key: KeyCode::G,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('g');
        }
        Event::KeyPressed {
            key: KeyCode::H,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('h');
        }
        Event::KeyPressed {
            key: KeyCode::I,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('i');
        }
        Event::KeyPressed {
            key: KeyCode::J,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('j');
        }
        Event::KeyPressed {
            key: KeyCode::K,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('k');
        }
        Event::KeyPressed {
            key: KeyCode::L,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('l');
        }
        Event::KeyPressed {
            key: KeyCode::M,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('m');
        }
        Event::KeyPressed {
            key: KeyCode::N,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('n');
        }
        Event::KeyPressed {
            key: KeyCode::O,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('o');
        }
        Event::KeyPressed {
            key: KeyCode::P,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('p');
        }
        Event::KeyPressed {
            key: KeyCode::Q,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('q');
        }
        Event::KeyPressed {
            key: KeyCode::R,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('r');
        }
        Event::KeyPressed {
            key: KeyCode::S,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('s');
        }
        Event::KeyPressed {
            key: KeyCode::T,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('t');
        }
        Event::KeyPressed {
            key: KeyCode::U,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('u');
        }
        Event::KeyPressed {
            key: KeyCode::V,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('v');
        }
        Event::KeyPressed {
            key: KeyCode::W,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('w');
        }
        Event::KeyPressed {
            key: KeyCode::X,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('x');
        }
        Event::KeyPressed {
            key: KeyCode::Y,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('y');
        }
        Event::KeyPressed {
            key: KeyCode::Z,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('z');
        }
        Event::KeyPressed {
            key: KeyCode::Row1,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('1');
        }
        Event::KeyPressed {
            key: KeyCode::Row2,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('2');
        }
        Event::KeyPressed {
            key: KeyCode::Row3,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('3');
        }
        Event::KeyPressed {
            key: KeyCode::Row4,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('4');
        }
        Event::KeyPressed {
            key: KeyCode::Row5,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('5');
        }
        Event::KeyPressed {
            key: KeyCode::Row6,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('6');
        }
        Event::KeyPressed {
            key: KeyCode::Row7,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('7');
        }
        Event::KeyPressed {
            key: KeyCode::Row8,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('8');
        }
        Event::KeyPressed {
            key: KeyCode::Row9,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('9');
        }
        Event::KeyPressed {
            key: KeyCode::Row0,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('0');
        }
        Event::KeyPressed {
            key: KeyCode::Grave,
            ctrl: _,
            shift: false,
        } => {
            state.command.push('`');
        }
        Event::KeyPressed {
            key: KeyCode::Minus,
            ctrl: _,
            shift: _,
        } => {
            state.command.push('-');
        }
        Event::KeyPressed {
            key: KeyCode::Equals,
            ctrl: _,
            shift: _,
        } => {
            state.command.push('=');
        }
        Event::KeyPressed {
            key: KeyCode::LeftBracket,
            ctrl: _,
            shift: _,
        } => {
            state.command.push('[');
        }
        Event::KeyPressed {
            key: KeyCode::RightBracket,
            ctrl: _,
            shift: _,
        } => {
            state.command.push(']');
        }
        Event::KeyPressed {
            key: KeyCode::Backslash,
            ctrl: _,
            shift: _,
        } => {
            state.command.push('\\');
        }
        Event::KeyPressed {
            key: KeyCode::Semicolon,
            ctrl: _,
            shift: _,
        } => {
            state.command.push(';');
        }
        Event::KeyPressed {
            key: KeyCode::Apostrophe,
            ctrl: _,
            shift: _,
        } => {
            state.command.push('\'');
        }
        Event::KeyPressed {
            key: KeyCode::Comma,
            ctrl: _,
            shift: _,
        } => {
            state.command.push(',');
        }
        Event::KeyPressed {
            key: KeyCode::Period,
            ctrl: _,
            shift: _,
        } => {
            state.command.push('.');
        }
        Event::KeyPressed {
            key: KeyCode::Slash,
            ctrl: _,
            shift: _,
        } => {
            state.command.push('/');
        }
        _ => (),
    }
}
