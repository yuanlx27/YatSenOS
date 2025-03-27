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
    let mut pos: usize = 0;
    let mut line = String::with_capacity(128);
    loop {
        // Print the prompt line.
        print!("\r\x1B[K> {line}");
        // Print the cursor (with offset for "> ").
        print!("\r\x1B[{}C", pos + 2);

        match pop_key() {
            0x0A | 0x0D => { // break on newline (LF|CR)
                print!("\n");
                break
            }
            0x08 | 0x7F => { // backspace (BS|DEL)
                if pos > 0 {
                    line.remove(pos - 1);
                    pos -= 1;
                }
            }
            0x1B => { // escape
                // Skip a '['.
                let _ = pop_key();

                match pop_key() {
                    0x43 => pos += (pos < line.len()) as usize,
                    0x44 => pos = pos.saturating_sub(1),
                    _ => {}
                }
            }
            c => {
                line.insert(pos, c as char);
                pos += 1;
            }
        }
    }
    line
}
