



pub fn find_cap_file(label: &str) -> crate::Result<String> {
    let path = format!("../capabilities/{}.json", label);
    if std::fs::exists(path.clone())? {
        return Ok(path);
    }
    Ok(format!("../capabilities/{}.json", "default"))
}