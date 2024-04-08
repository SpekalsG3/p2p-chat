fn main() {
    let mut buf = "Hello world".as_bytes().to_vec();
    buf.extend(vec![0; 256-buf.len()]);

    let str = String::from_utf8_lossy(&buf);
    println!("buf '{}'", str);
}
