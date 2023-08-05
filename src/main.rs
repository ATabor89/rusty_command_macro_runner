use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::Receiver,
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};

use windows::Win32::Foundation::POINT;

use serde::{Deserialize, Serialize};

mod keys;
use keys::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MacroConfig {
    program_hotkey: HashSet<Key>,
    macros: Vec<Macro>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Macro {
    macro_name: String,
    macro_hotkey: HashSet<Key>,
    commands: Vec<Command>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Command {
    GetMousePos,
    SetMousePos(i32, i32),
    LeftClick,
    MiddleClick,
    RightClick,
    PressKey(Key),
    PressKeyCombo(HashSet<Key>),
    TextInput(String), // TODO: Further validate functionality
    Wait(u64),
    Loop(u32, Vec<Self>),
}

impl Command {
    fn execute(&self) -> Result<(), anyhow::Error> {
        match self {
            Command::GetMousePos => {
                let point = get_cursor_pos()?;
                println!("{:?}", point);
            }
            Command::SetMousePos(x, y) => set_cursor_pos(*x, *y)?,
            Command::LeftClick => left_click()?,
            Command::MiddleClick => middle_click()?,
            Command::RightClick => right_click()?,
            Command::PressKey(key) => press_key(*key as i32)?,
            Command::PressKeyCombo(keys) => {
                press_key_combo(keys)?;
            }
            Command::Wait(wait_time_millis) => sleep(Duration::from_millis(*wait_time_millis)),
            Command::Loop(iterations, commands) => {
                match iterations {
                    0 => loop {
                        for command in commands.iter() {
                            command.execute()?;
                        }
                    },
                    _ => {
                        for _ in 0..*iterations {
                            for command in commands.iter() {
                                command.execute()?;
                            }
                        }
                    }
                };
            }
            Command::TextInput(text) => {
                for c in text.chars() {
                    if c.is_uppercase() {
                        press_key_combo(&[Key::Shift, Key::from(c)].into())?;
                    } else {
                        press_key(Key::from(c) as i32)?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(windows)]
fn get_cursor_pos() -> Result<POINT, anyhow::Error> {
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let mut point = POINT::default();

    if unsafe { GetCursorPos(&mut point) }.as_bool() {
        Ok(point)
    } else {
        Err(anyhow::anyhow!(
            "Failed to get cursor position: {}",
            get_last_windows_error()
        ))
    }
}

#[cfg(windows)]
fn set_cursor_pos(x: i32, y: i32) -> Result<(), anyhow::Error> {
    use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;

    if !unsafe { SetCursorPos(x, y) }.as_bool() {
        return Err(anyhow::anyhow!(
            "Failed to set cursor position: {}",
            get_last_windows_error()
        ));
    }

    Ok(())
}

#[cfg(windows)]
fn press_key_combo(keys: &HashSet<Key>) -> Result<(), anyhow::Error> {
    for key in keys.iter() {
        key_down(*key as i32)?;
    }

    for key in keys.iter() {
        key_up(*key as i32)?;
    }

    Ok(())
}

#[cfg(windows)]
fn get_last_windows_error() -> u32 {
    unsafe { windows::Win32::Foundation::GetLastError().0 }
}

#[cfg(windows)]
fn left_click() -> anyhow::Result<(), anyhow::Error> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    };

    let mut input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0::default(),
    };

    let mut mouse_input = unsafe { &mut input.Anonymous.mi };
    mouse_input.dwFlags = MOUSEEVENTF_LEFTDOWN;

    if unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) } != 1 {
        return Err(anyhow::anyhow!(
            "Failed to send mouse left down: {}",
            get_last_windows_error()
        ));
    }

    let mut mouse_input = unsafe { &mut input.Anonymous.mi };
    mouse_input.dwFlags = MOUSEEVENTF_LEFTUP;

    if unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) } != 1 {
        return Err(anyhow::anyhow!(
            "Failed to send mouse left up: {}",
            get_last_windows_error()
        ));
    }

    Ok(())
}

#[cfg(windows)]
fn middle_click() -> anyhow::Result<(), anyhow::Error> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
    };

    let mut input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0::default(),
    };

    let mut mouse_input = unsafe { &mut input.Anonymous.mi };
    mouse_input.dwFlags = MOUSEEVENTF_MIDDLEDOWN;

    if unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) } != 1 {
        return Err(anyhow::anyhow!(
            "Failed to send mouse middle down: {}",
            get_last_windows_error()
        ));
    }

    let mut mouse_input = unsafe { &mut input.Anonymous.mi };
    mouse_input.dwFlags = MOUSEEVENTF_MIDDLEUP;

    if unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) } != 1 {
        return Err(anyhow::anyhow!(
            "Failed to send mouse middle up: {}",
            get_last_windows_error()
        ));
    }

    Ok(())
}

#[cfg(windows)]
fn right_click() -> anyhow::Result<(), anyhow::Error> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    };

    let mut input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0::default(),
    };

    let mut mouse_input = unsafe { &mut input.Anonymous.mi };
    mouse_input.dwFlags = MOUSEEVENTF_RIGHTDOWN;

    if unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) } != 1 {
        return Err(anyhow::anyhow!(
            "Failed to send mouse right down: {}",
            get_last_windows_error()
        ));
    }

    let mut mouse_input = unsafe { &mut input.Anonymous.mi };
    mouse_input.dwFlags = MOUSEEVENTF_RIGHTUP;

    if unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) } != 1 {
        return Err(anyhow::anyhow!(
            "Failed to send mouse right up: {}",
            get_last_windows_error()
        ));
    }

    Ok(())
}

#[cfg(windows)]
fn press_key(key: i32) -> anyhow::Result<(), anyhow::Error> {
    key_down(key)?;
    key_up(key)?;

    Ok(())
}

#[cfg(windows)]
fn key_down(key: i32) -> anyhow::Result<(), anyhow::Error> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, VIRTUAL_KEY,
    };

    let mut input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0::default(),
    };

    let mut keyboard_input = unsafe { &mut input.Anonymous.ki };
    keyboard_input.wVk = VIRTUAL_KEY(key as u16);

    if unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) } != 1 {
        return Err(anyhow::anyhow!(
            "Failed to send key down for {}: {}",
            key,
            get_last_windows_error()
        ));
    }

    Ok(())
}

#[cfg(windows)]
fn key_up(key: i32) -> anyhow::Result<(), anyhow::Error> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYEVENTF_KEYUP, VIRTUAL_KEY,
    };

    let mut input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0::default(),
    };

    let mut keyboard_input = unsafe { &mut input.Anonymous.ki };
    keyboard_input.wVk = VIRTUAL_KEY(key as u16);

    keyboard_input.dwFlags.0 = KEYEVENTF_KEYUP.0;

    if unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) } != 1 {
        return Err(anyhow::anyhow!(
            "Failed to send key up for {}: {}",
            key,
            get_last_windows_error()
        ));
    }

    Ok(())
}

#[cfg(windows)]
fn key_pressed(vkey: i32) -> bool {
    (unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(vkey) } & 1 != 0)
}

#[cfg(windows)]
fn key_held(vkey: i32) -> bool {
    (unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(vkey) } & -0x8000i16
        != 0)
}

fn input_listener(macros: Vec<Macro>, rx: Receiver<Message>) -> Result<(), anyhow::Error> {
    let mut macro_threads: HashMap<usize, JoinHandle<()>> = HashMap::new();

    loop {
        if let Ok(Message::Exit) = rx.try_recv() {
            break;
        }

        for (index, current_macro) in macros.iter().enumerate() {
            if current_macro
                .macro_hotkey
                .iter()
                .all(|key| key_held(*key as i32) || key_pressed(*key as i32))
            {
                sleep(Duration::from_millis(1000));

                macro_threads
                    .entry(index)
                    .and_modify(|handle| {
                        if handle.is_finished() {
                            let commands = current_macro.commands.clone();

                            *handle = spawn(move || {
                                for command in commands.iter() {
                                    match command.execute() {
                                        Ok(_) => {}
                                        Err(e) => log::error!("Error: {}", e),
                                    }
                                }
                            });
                        } else {
                            log::warn!("Command already executing");
                            // TODO: Just warn or kill the thread?
                        }
                    })
                    .or_insert_with(|| {
                        let commands = current_macro.commands.clone();

                        spawn(move || {
                            for command in commands.iter() {
                                match command.execute() {
                                    Ok(_) => {}
                                    Err(e) => log::error!("Error: {}", e),
                                }
                            }
                        })
                    });
            }
        }

        sleep(Duration::from_millis(50));
    }

    Ok(())
}

enum Message {
    Exit,
}

fn main() -> Result<(), anyhow::Error> {
    // Initialize things
    // logger, config
    simple_logger::init_with_level(log::Level::Info)?;

    let macro_config_string = include_str!("../macro_config.yaml");
    let macro_config: MacroConfig = serde_yaml::from_str(macro_config_string)?;

    #[cfg(debug_assertions)]
    log::info!("{:#?}", macro_config);

    let (tx, rx) = std::sync::mpsc::channel();

    // Spawn a worker thread that acts as an input listener and executes the macros
    let input_listener_handle = spawn(move || input_listener(macro_config.macros, rx));

    loop {
        // If program_hotkey is pressed, exit program
        if macro_config
            .program_hotkey
            .iter()
            .all(|key| key_held(*key as i32))
        {
            tx.send(Message::Exit)?;
            break;
        }

        sleep(Duration::from_millis(50));
    }

    match input_listener_handle.join() {
        Ok(_) => {
            log::info!("Input listener thread exited")
        }
        Err(e) => log::error!("Error: {:?}", e),
    }

    Ok(())
}
