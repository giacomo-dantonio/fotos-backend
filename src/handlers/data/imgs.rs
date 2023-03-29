use std::path::PathBuf;

use tokio::io::AsyncRead;

pub fn is_image(filepath: &PathBuf) -> bool {
    unimplemented!()
}

static X: [u8; 1] = [0u8];

pub fn resize(filepath: &PathBuf, max_width: Option<u32>, max_height: Option<u32>) -> impl AsyncRead {
    X.as_slice()
}