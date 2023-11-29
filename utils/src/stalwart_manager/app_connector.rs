use serde::{de::DeserializeOwned, Serialize};

/// A way of talking to Stalwart and telling it what to do even though its a different process
pub trait AppConnection {
    type Config: DeserializeOwned + Serialize + Default;

    fn new(config: Self::Config) -> Self;
    fn restart(&self) -> Result<(), ()>;

    fn get_pid(&self) -> Result<u32, ()>;
}

pub mod none {
    use super::AppConnection;
    /// You are out of luck. We don't support your OS
    #[derive(Debug)]
    pub struct NoneConnection;

    impl AppConnection for NoneConnection {
        type Config = ();

        fn new(_: Self::Config) -> Self {
            Self
        }

        fn restart(&self) -> Result<(), ()> {
            Ok(())
        }

        fn get_pid(&self) -> Result<u32, ()> {
            Ok(0)
        }
    }
}

pub mod linux_connection {
    use std::fmt::Debug;

    use serde::{Deserialize, Serialize};

    use super::AppConnection;
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct LinuxConnectionConfig {
        pub service_path: String,
        pub systemctl_path: String,
    }
    impl Default for LinuxConnectionConfig {
        fn default() -> Self {
            Self {
                service_path: "/etc/systemd/system/stalwart.service".to_string(),
                systemctl_path: "/bin/systemctl".to_string(),
            }
        }
    }
    pub struct LinuxConnection {
        pub service_path: String,
        pub systemctl_path: String,
    }
    impl Debug for LinuxConnection {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let pid = self.get_pid().unwrap_or(0);
            f.debug_struct("LinuxConnection")
                .field("service_path", &self.service_path)
                .field("systemctl_path", &self.systemctl_path)
                .field("pid(If 0 it was not able to be retrieved)", &pid)
                .finish()
        }
    }
    impl AppConnection for LinuxConnection {
        type Config = LinuxConnectionConfig;

        fn new(config: Self::Config) -> Self {
            Self {
                service_path: config.service_path,
                systemctl_path: config.systemctl_path,
            }
        }

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
                    let output = String::from_utf8(output.stdout).map_err(|e| {
                        log::error!("Failed to parse output: {:?}", e);
                        ()
                    })?;
                    parse_pid_response(output)
                }
                Err(err) => {
                    log::error!("Failed to get pid: {:?}", err);
                    Err(())
                }
            }
        }
    }

    fn parse_pid_response(output: String) -> Result<u32, ()> {
        let output = output.trim();
        let output = output.split('=').collect::<Vec<&str>>();
        let output = output[1].parse::<u32>().unwrap();
        Ok(output)
    }

    #[test]
    fn test_pid_parse() {
        let output = "MainPID=1234";
        let output = parse_pid_response(output.to_string()).unwrap();
        assert_eq!(output, 1234);
    }
}
