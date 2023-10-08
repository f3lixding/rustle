pub enum Response {
    Hit(Vec<u8>),
    Miss(u16),
}
