use anyhow::bail;
use std::io::Read;
use std::os::unix::net::UnixStream;

pub fn as_bytes(v: &mut [u32]) -> &mut [u8] {
    unsafe {
        let (_prefix, result, _suffix) = v.align_to_mut::<u8>();
        return result;
    }
}

/// Returns number of u32's that were read from the stream.
pub fn read_input(mut stream: &UnixStream, buffer: &mut [u32]) -> anyhow::Result<usize> {
    let n = stream.read(&mut as_bytes(buffer)[..])?;
    if n % 4 != 0 {
        bail!("invalid msg");
    }
    return Ok(n/4);
}
