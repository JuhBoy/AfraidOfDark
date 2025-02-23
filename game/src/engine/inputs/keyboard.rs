use std::collections::HashMap;

#[derive(Debug)]
pub struct Keyboard {
    keys_mapping: HashMap<glfw::Key, usize>,
    keys_state_lookup: [i32; 117],
}

fn is_release(state: i32) -> bool {
    return state & (1 << 1) != 0;
}

fn is_pressed(state: i32) -> bool {
    return (state & 1) != 0;
}

fn compute_state(action: glfw::Action, _modifier: glfw::Modifiers) -> i32 {
    let mut result: i32 = 0;
    if action == glfw::Action::Press {
        result = result | 1;
    } else if action == glfw::Action::Release {
        result = result | (1 << 1);
    }

    // @todo: update modifier state here

    result
}

impl Keyboard {
    pub fn new() -> Keyboard {
        let new_keyboard = Keyboard {
            keys_state_lookup: std::array::from_fn(|_size| 0),
            keys_mapping: HashMap::from([
                (glfw::Key::Space, 0),
                (glfw::Key::Apostrophe, 1),
                (glfw::Key::Comma, 2),
                (glfw::Key::Minus, 3),
                (glfw::Key::Period, 4),
                (glfw::Key::Slash, 5),
                (glfw::Key::Num0, 6),
                (glfw::Key::Num1, 7),
                (glfw::Key::Num2, 8),
                (glfw::Key::Num3, 9),
                (glfw::Key::Num4, 10),
                (glfw::Key::Num5, 11),
                (glfw::Key::Num6, 12),
                (glfw::Key::Num7, 13),
                (glfw::Key::Num8, 14),
                (glfw::Key::Num9, 15),
                (glfw::Key::Semicolon, 16),
                (glfw::Key::Equal, 17),
                (glfw::Key::A, 18),
                (glfw::Key::B, 19),
                (glfw::Key::C, 20),
                (glfw::Key::D, 21),
                (glfw::Key::E, 22),
                (glfw::Key::F, 23),
                (glfw::Key::G, 24),
                (glfw::Key::H, 25),
                (glfw::Key::I, 26),
                (glfw::Key::J, 27),
                (glfw::Key::K, 28),
                (glfw::Key::L, 29),
                (glfw::Key::M, 30),
                (glfw::Key::N, 31),
                (glfw::Key::O, 32),
                (glfw::Key::P, 33),
                (glfw::Key::Q, 34),
                (glfw::Key::R, 35),
                (glfw::Key::S, 36),
                (glfw::Key::T, 37),
                (glfw::Key::U, 38),
                (glfw::Key::V, 39),
                (glfw::Key::LeftBracket, 40),
                (glfw::Key::Backslash, 41),
                (glfw::Key::RightBracket, 42),
                (glfw::Key::GraveAccent, 43),
                (glfw::Key::World1, 44),
                (glfw::Key::World2, 45),
                (glfw::Key::Escape, 46),
                (glfw::Key::Enter, 47),
                (glfw::Key::Tab, 48),
                (glfw::Key::Backspace, 49),
                (glfw::Key::Insert, 50),
                (glfw::Key::Delete, 51),
                (glfw::Key::Right, 52),
                (glfw::Key::Left, 53),
                (glfw::Key::Down, 54),
                (glfw::Key::Up, 55),
                (glfw::Key::PageUp, 56),
                (glfw::Key::PageDown, 57),
                (glfw::Key::Home, 58),
                (glfw::Key::End, 59),
                (glfw::Key::CapsLock, 60),
                (glfw::Key::ScrollLock, 61),
                (glfw::Key::NumLock, 62),
                (glfw::Key::PrintScreen, 63),
                (glfw::Key::Pause, 64),
                (glfw::Key::F1, 65),
                (glfw::Key::F2, 66),
                (glfw::Key::F3, 67),
                (glfw::Key::F4, 68),
                (glfw::Key::F5, 69),
                (glfw::Key::F6, 70),
                (glfw::Key::F7, 71),
                (glfw::Key::F8, 72),
                (glfw::Key::F9, 73),
                (glfw::Key::F10, 74),
                (glfw::Key::F11, 75),
                (glfw::Key::F12, 76),
                (glfw::Key::F13, 77),
                (glfw::Key::F14, 78),
                (glfw::Key::F15, 79),
                (glfw::Key::F16, 80),
                (glfw::Key::F17, 81),
                (glfw::Key::F18, 82),
                (glfw::Key::F19, 83),
                (glfw::Key::F20, 84),
                (glfw::Key::F21, 85),
                (glfw::Key::F22, 86),
                (glfw::Key::F23, 87),
                (glfw::Key::F24, 88),
                (glfw::Key::F25, 89),
                (glfw::Key::Kp0, 90),
                (glfw::Key::Kp1, 91),
                (glfw::Key::Kp2, 92),
                (glfw::Key::Kp3, 93),
                (glfw::Key::Kp4, 94),
                (glfw::Key::Kp5, 95),
                (glfw::Key::Kp6, 96),
                (glfw::Key::Kp7, 97),
                (glfw::Key::Kp8, 98),
                (glfw::Key::Kp9, 99),
                (glfw::Key::KpDecimal, 100),
                (glfw::Key::KpDivide, 101),
                (glfw::Key::KpMultiply, 102),
                (glfw::Key::KpSubtract, 103),
                (glfw::Key::KpAdd, 104),
                (glfw::Key::KpEnter, 105),
                (glfw::Key::KpEqual, 106),
                (glfw::Key::LeftShift, 107),
                (glfw::Key::LeftControl, 108),
                (glfw::Key::LeftAlt, 109),
                (glfw::Key::LeftSuper, 110),
                (glfw::Key::RightShift, 111),
                (glfw::Key::RightControl, 112),
                (glfw::Key::RightAlt, 113),
                (glfw::Key::RightSuper, 114),
                (glfw::Key::Menu, 115),
                (glfw::Key::Unknown, 116),
            ]),
        };

        new_keyboard
    }

    // Occures once at the beggining of the frame it is marked as released by the window system
    pub fn is_key_released(&self, key: glfw::Key) -> bool {
        if !self.keys_mapping.contains_key(&key) {
            println!(
                "[Inputs Keyboard] Could not find a mapping for the key {:?}",
                &key
            );
            return false;
        }

        let key_index: usize = *self.keys_mapping.get(&key).unwrap();
        let state: i32 = self.keys_state_lookup[key_index];

        is_release(state)
    }

    pub fn is_key_pressed(&self, key: glfw::Key) -> bool {
        if !self.keys_mapping.contains_key(&key) {
            println!(
                "[Inputs Keyboard] Could not find a mapping for the key {:?}",
                &key
            );
            return false;
        }

        let key_index: usize = *self.keys_mapping.get(&key).unwrap();
        let state: i32 = self.keys_state_lookup[key_index];

        is_pressed(state)
    }

    pub fn update_key_state(
        &mut self,
        key: glfw::Key,
        action: glfw::Action,
        modifier: glfw::Modifiers,
    ) {
        if !self.keys_mapping.contains_key(&key) {
            println!(
                "[Inputs Keyboard] Mapping doesn't contains the given key {:?}",
                &key
            );
            return;
        }

        // Access the memory addresse of the key slot and update the value
        let index: usize = *self.keys_mapping.get(&key).unwrap();
        let key_slot: &mut i32 = &mut self.keys_state_lookup[index];
        *key_slot = compute_state(action, modifier);
    }

		// Clear states like "released" at frame N+1 before new states occures
    pub fn pre_update_states(&mut self) {
        for key in self.keys_state_lookup.as_mut() {
            if is_release(*key) {
                let release_idx: i32 = 1 << 1;
                *key = *key & !release_idx;
            }
        }
    }
}
