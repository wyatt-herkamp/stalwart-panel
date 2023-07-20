use std::io;
use std::path::PathBuf;
/// A way of talking to Stalwart and telling it what to do even though its a different process
pub trait AppConnection {
    fn restart(&self) -> Result<(), ()>;

    fn get_pid(&self) -> Result<u32, ()>;

}

pub mod none {
    use super::AppConnection;
    /// You are out of luck. We don't support your OS
    #[derive(Debug)]
    pub struct NoneConnection;

    impl AppConnection for NoneConnection {
        fn restart(&self) -> Result<(), ()> {
            Ok(())
        }

        fn get_pid(&self) -> Result<u32, ()> {
            Ok(0)
        }
    }
}

pub mod linux_connection {
    use super::AppConnection;

    #[derive(Debug)]
    pub struct LinuxConnection {
        pub service_path: String,
        pub systemctl_path: String,
    }
    impl AppConnection for LinuxConnection {
        fn restart(&self) -> Result<(), ()> {
            let status = std::process::Command::new(&self.systemctl_path)
                .arg("restart")
                .arg(&self.service_path)
                .status();
            match status {
                Ok(code) => {
                    if code.success() {
                        Ok(())
                    } else {
                        Err(())
                    }
                }
                Err(_) => Err(()),
            }
        }

        fn get_pid(&self) -> Result<u32, ()> {
            let output = std::process::Command::new(&self.systemctl_path)
                .arg("show")
                .arg(&self.service_path)
                .arg("--property")
                .arg("MainPID")
                .output();
            match output {
                Ok(output) => {
                    let output = String::from_utf8(output.stdout).unwrap();
                    let output = output.trim();
                    let output = output.split('=').collect::<Vec<&str>>();
                    let output = output[1].parse::<u32>().unwrap();
                    Ok(output)
                }
                Err(_) => Err(()),
            }

        }
    }
}
