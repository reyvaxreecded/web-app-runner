#[cfg(windows)]
use std::os::windows::process::CommandExt;

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{window, Emitter, State};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationState {
    Running,
    Stopped,
    Errored,
}

impl Application {
    pub fn new(name: String, display_name: String, start_command: String, dir: String) -> Self {
        Self {
            name,
            display_name,
            start_command,
            dir,
            state: ApplicationState::Stopped,
            port: None,
            pid: None,
            logs: Vec::new(),
        }
    }

    pub fn start(&mut self, window: tauri::Window) -> Result<String, String> {
        // Start the application
        if self.state == ApplicationState::Running {
            self.logs.push("Application is already running".to_string());
            return Err(format!(
                "Le service {} est déjà en cours d'exécution",
                &self.name
            ));
        }

        let app = Arc::new(Mutex::new(self.clone()));
        let window_clone = Arc::new(window);

        let thr = thread::spawn({
            let app = Arc::clone(&app);
            let window_clone = Arc::clone(&window_clone);

            move || {
                let mut app = app.lock().unwrap();
                let mut command = if cfg!(target_os = "windows") {
                    let mut cmd = Command::new("cmd");
                    cmd.arg("/C")
                        .arg(&app.start_command)
                        .current_dir(&app.dir)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());
                    #[cfg(windows)]
                    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
                    cmd
                } else {
                    let mut cmd = Command::new("sh");
                    cmd.arg("-c")
                        .arg(&app.start_command)
                        .current_dir(&app.dir)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());
                    cmd
                };

                match command.spawn() {
                    Ok(mut child) => {
                        app.pid = Some(child.id());
                        app.logs
                            .push(format!("Started application with PID: {}", child.id()));
                        app.state = ApplicationState::Running;
                        window_clone
                            .emit("service-status-changed", &app.name)
                            .unwrap();
                        app.listen_logs(&mut child, window_clone.clone());
                        Ok(format!("Started application with PID: {}", child.id()))
                    }
                    Err(e) => {
                        app.state = ApplicationState::Errored;
                        app.logs.push(format!("Failed to start application: {}", e));
                        app.pid = None;
                        Err(format!("Failed to start application: {}", e.to_string()))
                    }
                }
            }
        });
        match thr.join() {
            Ok(result) => match result {
                Ok(_) => Ok("Application started successfully".to_string()),
                Err(e) => Err(format!("Error: {}", e)),
            },
            Err(e) => Err(format!("Fatal Thread error")),
        }
    }

    pub fn stop(&mut self) -> Result<String, String> {
        if self.state != ApplicationState::Running {
            return Err(format!(
                "Le service {} n'est pas en cours d'exécution",
                &self.name
            ));
        }

        if self.port != None && !cfg!(target_os = "windows") {
            let cmd = Command::new("lsof")
                .arg("ti")
                .arg(format!(":{}", &self.port.unwrap()))
                .stdout(Stdio::piped())
                .output();
            match cmd {
                Ok(output) => {
                    if let Ok(pid) = String::from_utf8(output.stdout) {
                        let pid = pid.trim().parse::<u32>().unwrap();
                        let kill_cmd = Command::new("kill").arg("-9").arg(pid.to_string()).output();
                        match kill_cmd {
                            Ok(_) => {
                                self.logs.push(format!("Killed process with PID: {}", pid));
                                Ok("Process killed successfully".to_string())
                            }
                            Err(e) => {
                                self.logs.push(format!("Failed to kill process: {}", e));
                                return Err(format!("Failed to kill process with PID: {}", pid));
                            }
                        }
                    } else {
                        self.logs.push("Failed to parse PID".to_string());
                        Err("Failed to parse PID".to_string())
                    }
                }
                Err(e) => {
                    self.logs.push(format!("Failed to get PID: {}", e));
                    return Err(format!("Failed to get PID for port {}: {}", &self.port, e));
                }
            }
        } else {
            Ok("Service stopped successfully".to_string())
        }
    }

    fn listen_logs(&mut self, child: &mut Child, window: Arc<tauri::Window>) {
        if let Some(mut output) = child.stdout.take() {
            let reader = BufReader::new(output);
            for line in reader.lines() {
                match line {
                    Ok(log) => {
                        self.logs.push(log.clone());
                        window.emit("service-log", log).unwrap();
                    }
                    Err(e) => {
                        self.logs.push(format!("Error reading log: {}", e));
                    }
                }
            }
        }
        if let Some(mut error) = child.stderr.take() {
            let reader = BufReader::new(error);
            for line in reader.lines() {
                match line {
                    Ok(log) => {
                        self.logs.push(log.clone());
                        window.emit("service-log", log).unwrap();
                    }
                    Err(e) => {
                        self.logs.push(format!("Error reading log: {}", e));
                    }
                }
            }
        }
    }
}
