use std::fmt;

fn main() -> Result<(), NoSpaceLeft> {
    let mut rb = create(3);

    write(&mut rb, "ab")?;

    write(&mut rb, "c")?;

    println!("{:?}", rb);
    println!("Read: {} , Buffer: {:?}", read(&mut rb, 1).unwrap(), rb);

    write(&mut rb, "e")?;
    println!("{:?}", rb);
    println!("Read: {} , Buffer: {:?}", read(&mut rb, 2).unwrap(), rb);

    match write(&mut rb, "fgz") {
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
    read_idx: usize,
    write_idx: usize,
    data: Vec<u8>,
}

#[derive(Debug, PartialEq)]
struct NoSpaceLeft;

impl fmt::Display for NoSpaceLeft {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No space in buffer")
    }
}

fn create(size: usize) -> RingBuffer {
    let data = vec![0; size];

    RingBuffer {
        read_idx: 0,
        write_idx: 0,
        data,
    }
}

fn write(rb: &mut RingBuffer, string: &str) -> Result<usize, NoSpaceLeft> {
    for ch in string.bytes() {
        if rb.data[rb.write_idx] == 0 {
            rb.data[rb.write_idx] = ch;
        } else {
            return Err(NoSpaceLeft);
        }

        rb.write_idx = (rb.write_idx + 1) % rb.data.len();
    }

    Ok(rb.write_idx)
}

fn read(rb: &mut RingBuffer, read_count: usize) -> Option<String> {
    let mut vec = Vec::new();

    rb.write_idx = rb.read_idx;

    for _ in 0..read_count {
        let ch = rb.data[rb.read_idx];
        vec.push(ch);
        rb.data[rb.read_idx] = 0;
        rb.read_idx = (rb.read_idx + 1) % rb.data.len();
    }

    String::from_utf8(vec).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let mut rb = create(3);
        assert_eq!(rb.data.len(), 3);

        assert_eq!(write(&mut rb, "ab").unwrap(), 2);

        assert_eq!(write(&mut rb, "c").unwrap(), 0);

        assert_eq!(read(&mut rb, 1).unwrap(), "a");

        assert_eq!(write(&mut rb, "ec").unwrap_err(), NoSpaceLeft);

        // assert_eq!(read(&mut rb, 2), "bc");

        // assert_eq!(write(&mut rb, "fghjy"), 0);

        // assert_eq!(read(&mut rb, 3), "efg");
    }
}
