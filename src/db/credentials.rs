pub mod db {
    use rpassword::read_password;
    use std::fmt;
    use std::io::{stdin, stdout, Write};

    type Result<T> = std::result::Result<T, CredentialsError>;
    #[derive(Debug, Clone)]
    pub enum CredentialsError {
        LoginEntryError(String),
        PasswordEntryError(String),
    }
    impl fmt::Display for CredentialsError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let tp = match self {
                CredentialsError::LoginEntryError(str) => ("logging", str),
                CredentialsError::PasswordEntryError(str) => ("password", str),
            };
            write!(
                f,
                "CoSelPro Credentials Error: Failed to get user {}! {}",
                tp.0, tp.1
            )
        }
    }

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
        pub fn from_console_prompt() -> Result<Credentials> {
            println!("Issue CoSelPro connection credentials:");

            print!("login: ");
            let _ = stdout().flush();
            let mut login = String::new();
            match stdin().read_line(&mut login) {
                Ok(_) => {}
                Err(e) => return Err(CredentialsError::LoginEntryError(e.to_string())),
            };
            login = login.trim().to_lowercase().to_string();

            print!("password: ");
            let _ = stdout().flush();
            let pwd = match read_password() {
                Ok(pwd) => pwd,
                Err(e) => return Err(CredentialsError::PasswordEntryError(e.to_string())),
            };

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
