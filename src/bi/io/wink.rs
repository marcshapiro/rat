// windows-specific input

pub fn inkey(_echo: bool) -> Result<char, String> {
    let line = inline()?;
    if line.is_empty() { return Ok('\0'); }
    let cs: Vec<char> = line.chars().take(1).collect();
    Ok(cs[0])
}
