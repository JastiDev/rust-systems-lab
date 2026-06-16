pub fn consume(value: String) {
    println!("{value}");
}

pub fn inspect(value: &str) {
    println!("{value}");
}

pub fn append_suffix(value: &mut String) {
    value.push_str("-processed");
}
