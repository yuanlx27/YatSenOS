use alloc::string::String;
use crossbeam_queue::ArrayQueue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Pressed(u8),
    Released(u8),
}

lazy_static! {
    static ref INPUT_BUF: ArrayQueue<Key> = ArrayQueue::new(128);
}

#[inline]
pub fn push_key(key: Key) {
    if INPUT_BUF.push(key).is_err() {
        warn!("Input buffer is full. Dropping key '{:?}'", key);
    }
}

#[inline]
pub fn try_pop_key() -> Option<Key> {
    INPUT_BUF.pop()
}

#[inline]
pub fn pop_key() -> Key {
    loop {
        if let Some(key) = try_pop_key() {
            return key;
        }
    }
}

#[inline]
pub fn get_line() -> String {
    let mut line = String::with_capacity(128);
    loop {
        match pop_key() {
            Key::Pressed(b'\n') => {
                print!("\n");
                break
            }
            Key::Pressed(0x08) | Key::Pressed(0x7F) => {
                line.pop();
            }
            Key::Pressed(ch) => {
                line.push(ch as char);
            }
            _ => {}
        }
    }
    line
}
