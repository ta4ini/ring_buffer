use std::{
    fmt,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

fn main() -> Result<(), NoSpaceLeft> {
    let rb = RingBuffer::create(3);

    rb.write("ab")?;

    rb.write("c")?;

    println!("{:?}", rb);
    println!(
        "Read: {} , Len: {}, Buffer: {:?}",
        rb.read(1).unwrap(),
        rb.len(),
        rb
    );

    rb.write("e")?;
    println!("{:?}", rb);
    println!(
        "Read: {} , Len: {}, Buffer: {:?}",
        rb.read(2).unwrap(),
        rb.len(),
        rb
    );

    match rb.write("fgz") {
        Ok(value) => println!("Success: {}", value),
        Err(e) => println!("Error: {}", e),
    }
    // write(&mut rb, "fgz")?;
    println!("{:?}", rb);

    Ok(())
}

// Пример API, вызовов и как меняется состояние буффера:
// [ _ _ _ ] create(3)
// [ a b _ ] write "ab" -> return 2
// [ a b c ] write "cd" -> return 1
// [ _ b c ] read(1) -> return "a"
// [ e b c ] write "e" -> return 1
// [ e _ _ ] read(2) -> return "bc"
// Ваша задача написать такой буффер и добавить тесты

#[derive(Debug)]
struct RingBuffer {
    read_idx: AtomicUsize,
    write_idx: AtomicUsize,
    data: Mutex<Vec<u8>>,
}

#[derive(Debug, PartialEq)]
struct NoSpaceLeft;

impl fmt::Display for NoSpaceLeft {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No space in buffer")
    }
}

impl RingBuffer {
    fn create(size: usize) -> Self {
        Self {
            read_idx: 0.into(),
            write_idx: 0.into(),
            data: Mutex::new(vec![0; size]),
        }
    }

    fn write(&self, string: &str) -> Result<usize, NoSpaceLeft> {
        for ch in string.bytes() {
            let mut data = self.data.lock().unwrap();
            let write_idx = self.write_idx.load(Ordering::Acquire);
            if data[write_idx] == 0 {
                data[write_idx] = ch;
            } else {
                return Err(NoSpaceLeft);
            }

            self.write_idx
                .store((write_idx + 1) % data.len(), Ordering::Relaxed);
        }

        Ok(self.write_idx.load(Ordering::Relaxed))
    }

    fn read(&self, read_count: usize) -> Option<String> {
        let mut vec = Vec::new();

        let read_idx = self.read_idx.load(Ordering::Acquire);
        self.write_idx.store(read_idx, Ordering::Release);

        for _ in 0..read_count {
            let mut data = self.data.lock().unwrap();
            let ch = data[read_idx];
            vec.push(ch);
            data[read_idx] = 0;
            self.read_idx
                .store((read_idx + 1) % data.len(), Ordering::Release);
        }

        String::from_utf8(vec).ok()
    }

    fn len(&self) -> usize {
        self.data
            .lock()
            .unwrap()
            .iter()
            .filter(|p| **p != 0)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};
    use rand::Rng;
    use super::*;

    #[test]
    fn test_1() {
        let rb = RingBuffer::create(3);
        assert_eq!(rb.data.lock().unwrap().len(), 3);

        assert_eq!(rb.write("ab").unwrap(), 2);

        assert_eq!(rb.write("c").unwrap(), 0);

        assert_eq!(rb.read(1).unwrap(), "a");

        assert_eq!(rb.write("ec").unwrap_err(), NoSpaceLeft);
    }

    #[test]
    fn test_2() {
        let buf_size = 10;
        let buffer = Arc::new(RingBuffer::create(buf_size));

        let mut handles = vec![];
        for _ in 1..100 {
            let buffer_ref = Arc::clone(&buffer);

            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    if buf_size == buffer_ref.len() {
                        buffer_ref.write("a").unwrap();
                    } else {
                        let mut rng = rand::rng();
                        buffer_ref.read(rng.random_range(0..=buf_size)).unwrap();
                    }
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let len = buffer.len();

        assert!(len <= 10);
    }
}
