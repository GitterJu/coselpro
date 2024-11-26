
pub mod db {
    use std::error::Error;
    use std::io::{stdin, stdout, Write};
    use rpassword::read_password;

    #[derive(Debug)]
    pub struct Credentials {
        login: String,
        password: String,
    }
    impl Credentials {
        pub fn new(login: &str, password: &str) -> Credentials {
            Credentials {
                login: login.to_string(),
                password: password.to_string(),
            }
        }
        pub fn get_login(&self) -> &String {
            &self.login
        }
        pub fn get_password_md5(&self) -> String {
            let mut wrd: String = self.password.clone();
            wrd.push_str(&self.get_login());
            let mut mp = format!("{:x}", md5::compute(&wrd[..])).to_string();
            mp.insert_str(0, "md5");
            mp
        }
        /// Prompt user in console for login and password and returns a credential
        pub fn from_console_prompt() -> Result<Credentials, Box<dyn Error>> {
            println!("Issue CoSelPro connection credentials:");

            print!("login: ");
            let _ = stdout().flush();
            let mut login = String::new();
            stdin().read_line(&mut login)?;
            login = login.trim().to_lowercase().to_string();

            print!("password: ");
            let _ = stdout().flush();
            let pwd = read_password()?;

            let creds = Credentials::new(&login, &pwd.trim().to_string());
            Ok(creds)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::credentials::db::Credentials;

    #[test]
    fn get_password() {
        let cred = Credentials::new("consult", "consult");
        assert_eq!(
            cred.get_password_md5(),
            "md55e73b42456347af1be4be2d0c8eda64a"
        );
    }
}