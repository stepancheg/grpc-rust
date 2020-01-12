#[allow(dead_code)]
pub fn assert_send<T: Send>() {}
#[allow(dead_code)]
pub fn assert_sync<T: Sync>() {}
#[allow(dead_code)]
pub fn assert_unpin<T: Unpin>() {}
