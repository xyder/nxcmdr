pub const TOKEN_FILENAME: &str = "data1.bin";
pub const DATA_FILENAME: &str = "data2.bin";
pub const BW_API_BASE: &str = "https://vault.bitwarden.com/api";
pub const BW_IDENTITY_BASE: &str = "https://vault.bitwarden.com/identity";


#[macro_export]
macro_rules! PRELOGIN_URL {() => { format!("{base}/accounts/prelogin", base = BW_API_BASE) }}

#[macro_export]
macro_rules! TOKEN_URL {() => { format!("{base}/connect/token", base = BW_IDENTITY_BASE) }}

#[macro_export]
macro_rules! SYNC_URL {() => { format!("{base}/sync", base = BW_API_BASE) }}

#[macro_export]
macro_rules! REVISION_URL {() => { format!("{base}/accounts/revision-date", base = BW_API_BASE) }}
