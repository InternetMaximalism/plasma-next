use std::fmt::Display;

pub fn join_str_with_separator<T: Display>(slice: &[T], separator: &str) -> String {
    let mut it = slice.iter();
    let mut result = String::new();

    if let Some(first) = it.next() {
        result.push_str(&first.to_string());
    }

    for element in it {
        result.push_str(separator);
        result.push_str(&element.to_string());
    }

    result
}
