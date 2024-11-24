mod api;

mod api_test {
    use crate::api::connection;

    fn test_pronpt() {
        let cred = connection::prompt_console_credentials();
        println!("{:?}", cred);
    }
}
