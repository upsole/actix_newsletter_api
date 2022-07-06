#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod schema;
pub mod models; 


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
