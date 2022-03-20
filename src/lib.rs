
pub mod virtual_tun;
pub use virtual_tun::VirtualTunInterface;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
