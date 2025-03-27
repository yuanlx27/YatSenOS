use alloc::string::String;
use crossbeam_queue::ArrayQueue;

type Key = u8;

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
        print!("\r\x1B[K> {line}");

        match pop_key() {
            b'\n' | b'\r' => {
                print!("\n");
                break
            }
            0x08 | 0x7F => {
                line.pop();
            }
            c => {
                line.push(c as char);
            }
        }
    }
    line
}
