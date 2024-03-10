pub fn dot<T: std::fmt::Display>(s: T) -> ::askama::Result<String> {
    let mut s = s.to_string();
    for (from, to) in &[("\\", "\\\\"), ("\"", "\\\""), ("\n", "\\n")] {
        s = s.replace(from, to);
    }

    Ok(s)
}
