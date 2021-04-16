use std::fmt::Debug;

use bytes::buf::Chain;
use bytes::Bytes;
use sqlx_core::Result;

use crate::MySqlClientError;

mod caching_sha2;
mod dialog;
mod native;
mod rsa;
mod sha256;

pub(crate) use self::caching_sha2::CachingSha2AuthPlugin;
pub(crate) use self::dialog::DialogAuthPlugin;
pub(crate) use self::native::NativeAuthPlugin;
pub(crate) use self::sha256::Sha256AuthPlugin;

pub(crate) trait AuthPlugin: 'static + Debug + Send + Sync {
    fn name(&self) -> &'static str;

    // Invoke the auth plugin and return the auth response
    fn invoke(&self, nonce: &Chain<Bytes, Bytes>, password: &str) -> Vec<u8>;

    // Handle "more data" from the MySQL server
    //  which tells the plugin some plugin-specific information
    //  if the plugin returns Some(_) that is sent back to MySQL
    fn handle(
        &self,
        command: u8,
        data: Bytes,
        nonce: &Chain<Bytes, Bytes>,
        password: &str,
    ) -> Result<Option<Vec<u8>>>;
}

impl dyn AuthPlugin {
    pub(crate) fn parse(s: &str) -> Result<Box<Self>> {
        match s {
            _ if s == CachingSha2AuthPlugin.name() => Ok(Box::new(CachingSha2AuthPlugin)),
            _ if s == Sha256AuthPlugin.name() => Ok(Box::new(Sha256AuthPlugin)),
            _ if s == NativeAuthPlugin.name() => Ok(Box::new(NativeAuthPlugin)),
            _ if s == DialogAuthPlugin.name() => Ok(Box::new(DialogAuthPlugin)),

            _ => Err(MySqlClientError::UnknownAuthPlugin(s.to_owned()).into()),
        }
    }
}

// XOR(x, y)
// If len(y) < len(x), wrap around inside y
fn xor_eq(x: &mut [u8], y: &[u8]) {
    let y_len = y.len();

    for i in 0..x.len() {
        x[i] ^= y[i % y_len];
    }
}
