use base64::Engine;
use base64::engine::general_purpose::STANDARD as b64;

pub trait Base64 {
    fn to_base64(&self) -> String;
}

impl<T> Base64 for Vec<T>
where
    Vec<T>: AsRef<[u8]>,
{
    fn to_base64(&self) -> String {
        b64.encode(self)
    }
}

impl<const N: usize> Base64 for [u8; N] {
    fn to_base64(&self) -> String {
        b64.encode(self)
    }
}
