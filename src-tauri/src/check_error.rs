use surrealdb::Error;

fn main() {
    let _e = Error::Api("test".into());
}
