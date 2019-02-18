extern crate serde;
extern crate serde_json;
extern crate mirror;

pub mod client;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
