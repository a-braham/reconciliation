#![recursion_limit = "256"]

use std::env;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenvy::dotenv;

pub mod models;
pub mod schema;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    PgConnection::establish(&database_url).unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
