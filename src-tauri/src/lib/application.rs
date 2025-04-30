use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Application {
    name: String,
    display_name: String,
    start_command: String,
    dir: String,
    state: ApplicationState,
    port: Option<u16>,
    pid: Option<u32>,
    logs: Vec<String>,
}

pub enum ApplicationState {
    Running,
    Stopped,
    Errored,
}

impl Application {
    pub fn new(name: String, display_name: String, star_command: String, dir: String) -> Self {
        Self {
            name,
            display_name,
            star_command,
            dir,
            state: ApplicationState::Stopped,
            port: None,
            pid: None,
            logs: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        // Start the application
        self.state = ApplicationState::Running;
            #[cfg(windows)] {
                Command::new("cmd")
                .arg("/C")
                .arg(&self.start_command)
                .current_dir(&self.dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .spawn()
                .expect("Failed to start application");
            } 
            #[cfg(not(windows))] {
                Command::new("sh")
                .arg("-c")
                .arg(&self.start_command)
                .current_dir(&self.dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start application");
            }

    }

    pub fn stop(&mut self) {
        // Stop the application
        self.state = ApplicationState::Stopped;
    }

}