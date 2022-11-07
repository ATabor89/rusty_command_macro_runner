use std::{
    collections::{HashMap, HashSet},
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};

use windows::Win32::Foundation::POINT;

use serde::{Deserialize, Serialize};

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

fn input_listener(macros: Vec<Macro>) -> Result<(), anyhow::Error> {
    let mut macro_threads: HashMap<usize, JoinHandle<()>> = HashMap::new();

    loop {
        for (index, mcro) in macros.iter().enumerate() {
            if mcro
                .macro_hotkey
                .iter()
                .all(|key| key_held(*key as i32) || key_pressed(*key as i32))
            {
                sleep(Duration::from_millis(1000));

                match macro_threads.get(&index) {
                    Some(handle) => {
                        if handle.is_finished() {
                            let commands = mcro.commands.clone();

                            macro_threads.insert(
                                index,
                                spawn(move || {
                                    for command in commands.iter() {
                                        match command.execute() {
                                            Ok(_) => {}
                                            Err(e) => log::error!("Error: {}", e),
                                        }
                                    }
                                }),
                            );
                        } else {
                            log::warn!("Command already executing");
                            // TODO: nothing for now
                            // Kill thread?
                        }
                    }
                    None => {
                        let commands = mcro.commands.clone();

                        macro_threads.insert(
                            index,
                            spawn(move || {
                                for command in commands.iter() {
                                    command.execute().unwrap();
                                }
                            }),
                        );
                    }
                }
            }
        }

        sleep(Duration::from_millis(50));
    }
}

fn main() -> Result<(), anyhow::Error> {
    // Initialize things
    // logger, config
    simple_logger::init_with_level(log::Level::Info)?;

    let macro_config_string = include_str!("../macro_config.yaml");
    let macro_config: MacroConfig = serde_yaml::from_str(macro_config_string)?;

    #[cfg(debug_assertions)]
    log::info!("{:#?}", macro_config);

    // Spawn a worker thread that acts as an input listener and executes the macros
    spawn(move || input_listener(macro_config.macros));

    loop {
        // If program_hotkey is pressed, exit program
        if macro_config
            .program_hotkey
            .iter()
            .all(|key| key_held(*key as i32))
        {
            break;
        }

        sleep(Duration::from_millis(50));
    }

    Ok(())
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
enum Key {
    LeftButton = 0x01,
    RightButton = 0x02,
    Cancel = 0x03,
    MiddleButton = 0x04,
    XButton1 = 0x05,
    XButton2 = 0x06,
    Back = 0x08,
    Tab = 0x09,
    Clear = 0x0C,
    Return = 0x0D,
    Shift = 0x10,
    Control = 0x11,
    Menu = 0x12,
    Pause = 0x13,
    Capital = 0x14,
    // KANA =	0x15,
    // HANGUEL =	0x15,
    // HANGUL =	0x15,
    // IME_ON =	0x16,
    // JUNJA = 0x17,
    // FINAL = 0x18,
    // HANJA =	0x19,
    // KANJI =	0x19,
    // IME_OFF =	0x1A,
    Escape = 0x1B,
    Convert = 0x1C,
    NonConvert = 0x1D,
    Accept = 0x1E,
    ModeChange = 0x1F,
    Space = 0x20,
    Prior = 0x21,
    Next = 0x22,
    End = 0x23,
    Home = 0x24,
    Left = 0x25,
    Up = 0x26,
    Right = 0x27,
    Down = 0x28,
    Select = 0x29,
    Print = 0x2A,
    Execute = 0x2B,
    Snapshot = 0x2C,
    Insert = 0x2D,
    Delete = 0x2E,
    Help = 0x2F,
    Key0 = 0x30,
    Key1 = 0x31,
    Key2 = 0x32,
    Key3 = 0x33,
    Key4 = 0x34,
    Key5 = 0x35,
    Key6 = 0x36,
    Key7 = 0x37,
    Key8 = 0x38,
    Key9 = 0x39,
    A = 0x41,
    B = 0x42,
    C = 0x43,
    D = 0x44,
    E = 0x45,
    F = 0x46,
    G = 0x47,
    H = 0x48,
    I = 0x49,
    J = 0x4A,
    K = 0x4B,
    L = 0x4C,
    M = 0x4D,
    N = 0x4E,
    O = 0x4F,
    P = 0x50,
    Q = 0x51,
    R = 0x52,
    S = 0x53,
    T = 0x54,
    U = 0x55,
    V = 0x56,
    W = 0x57,
    X = 0x58,
    Y = 0x59,
    Z = 0x5A,
    LeftWindows = 0x5B,
    RightWindows = 0x5C,
    Applications = 0x5D,
    Sleep = 0x5F,
    Numpad0 = 0x60,
    Numpad1 = 0x61,
    Numpad2 = 0x62,
    Numpad3 = 0x63,
    Numpad4 = 0x64,
    Numpad5 = 0x65,
    Numpad6 = 0x66,
    Numpad7 = 0x67,
    Numpad8 = 0x68,
    Numpad9 = 0x69,
    Multiply = 0x6A,
    Add = 0x6B,
    Separator = 0x6C,
    Subtract = 0x6D,
    Decimal = 0x6E,
    Divide = 0x6F,
    F1 = 0x70,
    F2 = 0x71,
    F3 = 0x72,
    F4 = 0x73,
    F5 = 0x74,
    F6 = 0x75,
    F7 = 0x76,
    F8 = 0x77,
    F9 = 0x78,
    F10 = 0x79,
    F11 = 0x7A,
    F12 = 0x7B,
    F13 = 0x7C,
    F14 = 0x7D,
    F15 = 0x7E,
    F16 = 0x7F,
    F17 = 0x80,
    F18 = 0x81,
    F19 = 0x82,
    F20 = 0x83,
    F21 = 0x84,
    F22 = 0x85,
    F23 = 0x86,
    F24 = 0x87,
    Numlock = 0x90,
    Scroll = 0x91,
    Unassigned = 0x97,
    LeftShift = 0xA0,
    RightShift = 0xA1,
    LeftControl = 0xA2,
    RightControl = 0xA3,
    LeftMenu = 0xA4,
    RightMenu = 0xA5,
    BrowserBack = 0xA6,
    BrowserForward = 0xA7,
    BrowserRefresh = 0xA8,
    BrowserStop = 0xA9,
    BrowserSearch = 0xAA,
    BrowserFavorites = 0xAB,
    BrowserHome = 0xAC,
    VolumeMute = 0xAD,
    VolumeDown = 0xAE,
    VolumeUp = 0xAF,
    MediaNextTrack = 0xB0,
    MediaPrevTrack = 0xB1,
    MediaStop = 0xB2,
    MediaPlayPause = 0xB3,
    LaunchMail = 0xB4,
    LaunchMediaSelect = 0xB5,
    LaunchApp1 = 0xB6,
    LaunchApp2 = 0xB7,
    Oem1 = 0xBA,
    OemPlus = 0xBB,
    OemComma = 0xBC,
    OemMinus = 0xBD,
    OemPeriod = 0xBE,
    Oem2 = 0xBF,
    Oem3 = 0xC0,
    Oem4 = 0xDB,
    Oem5 = 0xDC,
    Oem6 = 0xDD,
    Oem7 = 0xDE,
    Oem8 = 0xDF,
    Oem102 = 0xE2,
    ProcessKey = 0xE5,
    Packet = 0xE7,
    Attn = 0xF6,
    CrSel = 0xF7,
    ExSel = 0xF8,
    EraseEOF = 0xF9,
    Play = 0xFA,
    Zoom = 0xFB,
    PA1 = 0xFD,
    OemClear = 0xFE,
}

impl From<char> for Key {
    fn from(c: char) -> Self {
        match c {
            ' ' => Key::Space,
            'A' | 'a' => Key::A,
            'B' | 'b' => Key::B,
            'C' | 'c' => Key::C,
            'D' | 'd' => Key::D,
            'E' | 'e' => Key::E,
            'F' | 'f' => Key::F,
            'G' | 'g' => Key::G,
            'H' | 'h' => Key::H,
            'I' | 'i' => Key::I,
            'J' | 'j' => Key::J,
            'K' | 'k' => Key::K,
            'L' | 'l' => Key::L,
            'M' | 'm' => Key::M,
            'N' | 'n' => Key::N,
            'O' | 'o' => Key::O,
            'P' | 'p' => Key::P,
            'Q' | 'q' => Key::Q,
            'R' | 'r' => Key::R,
            'S' | 's' => Key::S,
            'T' | 't' => Key::T,
            'U' | 'u' => Key::U,
            'V' | 'v' => Key::V,
            'W' | 'w' => Key::W,
            'X' | 'x' => Key::X,
            'Y' | 'y' => Key::Y,
            'Z' | 'z' => Key::Z,
            '0' => Key::Key0,
            '1' => Key::Key1,
            '2' => Key::Key2,
            '3' => Key::Key3,
            '4' => Key::Key4,
            '5' => Key::Key5,
            '6' => Key::Key6,
            '7' => Key::Key7,
            '8' => Key::Key8,
            '9' => Key::Key9,
            _ => Key::Unassigned,
        }
    }
}

impl From<i32> for Key {
    fn from(n: i32) -> Self {
        match n {
            0x01 => Key::LeftButton,
            0x02 => Key::RightButton,
            0x03 => Key::Cancel,
            0x04 => Key::MiddleButton,
            0x05 => Key::XButton1,
            0x06 => Key::XButton2,
            0x08 => Key::Back,
            0x09 => Key::Tab,
            0x0C => Key::Clear,
            0x0D => Key::Return,
            0x10 => Key::Shift,
            0x11 => Key::Control,
            0x12 => Key::Menu,
            0x13 => Key::Pause,
            0x14 => Key::Capital,
            // KANA =	0x15,
            // HANGUEL =	0x15,
            // HANGUL =	0x15,
            // IME_ON =	0x16,
            // JUNJA = 0x17,
            // FINAL = 0x18,
            // HANJA =	0x19,
            // KANJI =	0x19,
            // IME_OFF =	0x1A,
            0x1B => Key::Escape,
            0x1C => Key::Convert,
            0x1D => Key::NonConvert,
            0x1E => Key::Accept,
            0x1F => Key::ModeChange,
            0x20 => Key::Space,
            0x21 => Key::Prior,
            0x22 => Key::Next,
            0x23 => Key::End,
            0x24 => Key::Home,
            0x25 => Key::Left,
            0x26 => Key::Up,
            0x27 => Key::Right,
            0x28 => Key::Down,
            0x29 => Key::Select,
            0x2A => Key::Print,
            0x2B => Key::Execute,
            0x2C => Key::Snapshot,
            0x2D => Key::Insert,
            0x2E => Key::Delete,
            0x2F => Key::Help,
            0x30 => Key::Key0,
            0x31 => Key::Key1,
            0x32 => Key::Key2,
            0x33 => Key::Key3,
            0x34 => Key::Key4,
            0x35 => Key::Key5,
            0x36 => Key::Key6,
            0x37 => Key::Key7,
            0x38 => Key::Key8,
            0x39 => Key::Key9,
            0x41 => Key::A,
            0x42 => Key::B,
            0x43 => Key::C,
            0x44 => Key::D,
            0x45 => Key::E,
            0x46 => Key::F,
            0x47 => Key::G,
            0x48 => Key::H,
            0x49 => Key::I,
            0x4A => Key::J,
            0x4B => Key::K,
            0x4C => Key::L,
            0x4D => Key::M,
            0x4E => Key::N,
            0x4F => Key::O,
            0x50 => Key::P,
            0x51 => Key::Q,
            0x52 => Key::R,
            0x53 => Key::S,
            0x54 => Key::T,
            0x55 => Key::U,
            0x56 => Key::V,
            0x57 => Key::W,
            0x58 => Key::X,
            0x59 => Key::Y,
            0x5A => Key::Z,
            0x5B => Key::LeftWindows,
            0x5C => Key::RightWindows,
            0x5D => Key::Applications,
            0x5F => Key::Sleep,
            0x60 => Key::Numpad0,
            0x61 => Key::Numpad1,
            0x62 => Key::Numpad2,
            0x63 => Key::Numpad3,
            0x64 => Key::Numpad4,
            0x65 => Key::Numpad5,
            0x66 => Key::Numpad6,
            0x67 => Key::Numpad7,
            0x68 => Key::Numpad8,
            0x69 => Key::Numpad9,
            0x6A => Key::Multiply,
            0x6B => Key::Add,
            0x6C => Key::Separator,
            0x6D => Key::Subtract,
            0x6E => Key::Decimal,
            0x6F => Key::Divide,
            0x70 => Key::F1,
            0x71 => Key::F2,
            0x72 => Key::F3,
            0x73 => Key::F4,
            0x74 => Key::F5,
            0x75 => Key::F6,
            0x76 => Key::F7,
            0x77 => Key::F8,
            0x78 => Key::F9,
            0x79 => Key::F10,
            0x7A => Key::F11,
            0x7B => Key::F12,
            0x7C => Key::F13,
            0x7D => Key::F14,
            0x7E => Key::F15,
            0x7F => Key::F16,
            0x80 => Key::F17,
            0x81 => Key::F18,
            0x82 => Key::F19,
            0x83 => Key::F20,
            0x84 => Key::F21,
            0x85 => Key::F22,
            0x86 => Key::F23,
            0x87 => Key::F24,
            0x90 => Key::Numlock,
            0x91 => Key::Scroll,
            0x97 => Key::Unassigned,
            0xA0 => Key::LeftShift,
            0xA1 => Key::RightShift,
            0xA2 => Key::LeftControl,
            0xA3 => Key::RightControl,
            0xA4 => Key::LeftMenu,
            0xA5 => Key::RightMenu,
            0xA6 => Key::BrowserBack,
            0xA7 => Key::BrowserForward,
            0xA8 => Key::BrowserRefresh,
            0xA9 => Key::BrowserStop,
            0xAA => Key::BrowserSearch,
            0xAB => Key::BrowserFavorites,
            0xAC => Key::BrowserHome,
            0xAD => Key::VolumeMute,
            0xAE => Key::VolumeDown,
            0xAF => Key::VolumeUp,
            0xB0 => Key::MediaNextTrack,
            0xB1 => Key::MediaPrevTrack,
            0xB2 => Key::MediaStop,
            0xB3 => Key::MediaPlayPause,
            0xB4 => Key::LaunchMail,
            0xB5 => Key::LaunchMediaSelect,
            0xB6 => Key::LaunchApp1,
            0xB7 => Key::LaunchApp2,
            0xBA => Key::Oem1,
            0xBB => Key::OemPlus,
            0xBC => Key::OemComma,
            0xBD => Key::OemMinus,
            0xBE => Key::OemPeriod,
            0xBF => Key::Oem2,
            0xC0 => Key::Oem3,
            0xDB => Key::Oem4,
            0xDC => Key::Oem5,
            0xDD => Key::Oem6,
            0xDE => Key::Oem7,
            0xDF => Key::Oem8,
            0xE2 => Key::Oem102,
            0xE5 => Key::ProcessKey,
            0xE7 => Key::Packet,
            0xF6 => Key::Attn,
            0xF7 => Key::CrSel,
            0xF8 => Key::ExSel,
            0xF9 => Key::EraseEOF,
            0xFA => Key::Play,
            0xFB => Key::Zoom,
            0xFD => Key::PA1,
            0xFE => Key::OemClear,
            _ => {
                log::warn!("{} is not a valid key, mapping to Unassigned", n);
                Key::Unassigned
            }
        }
    }
}
