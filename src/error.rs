use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Device not connected")]
    NotConnected,
}
