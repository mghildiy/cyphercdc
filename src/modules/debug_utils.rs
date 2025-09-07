pub fn bytes_to_utfstring(bytes: &[u8]) -> Result<String, std::str::Utf8Error> {
    match std::str::from_utf8(bytes) {
        Ok(str) => Ok(str.to_owned()),
        Err(e) => Err(e)
    }
}