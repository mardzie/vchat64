use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid Socket Addr: {0}")]
    InvalidSocketAddr(#[from] std::io::Error),
}
