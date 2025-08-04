use std::process::Command;
use std::path::{PathBuf};
use std::io;

#[derive(Debug)]
pub enum SimulationEnvError {
    PythonNotFound,
    PythonVersionTooLow(String),
    PipInstallFailed(String),
    DependencyCheckFailed(String),
    SimulationPathNotFound,
    Io(io::Error),
}

impl From<io::Error> for SimulationEnvError {
    fn from(e: io::Error) -> Self {
        SimulationEnvError::Io(e)
    }
}

/// Detect the correct python command (cross-platform)
pub fn detect_python_cmd() -> Result<String, SimulationEnvError> {
    let candidates = if cfg!(windows) {
        vec!["python", "py", "python3"]
    } else {
        vec!["python3", "python"]
    };
    for cmd in candidates {
        if let Ok(output) = Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                return Ok(cmd.to_string());
            }
        }
    }
    Err(SimulationEnvError::PythonNotFound)
}

/// Check python version >= 3.8
pub fn check_python_version(python_cmd: &str) -> Result<(), SimulationEnvError> {
    let output = Command::new(python_cmd)
        .arg("--version")
        .output()
        .map_err(|_| SimulationEnvError::PythonNotFound)?;
    let version_str = String::from_utf8_lossy(&output.stdout).to_string() + &String::from_utf8_lossy(&output.stderr);
    let ver = version_str.trim().replace("Python ", "");
    let parts: Vec<&str> = ver.split('.').collect();
    if parts.len() >= 2 {
        let major = parts[0].parse::<u32>().unwrap_or(0);
        let minor = parts[1].parse::<u32>().unwrap_or(0);
        if major > 3 || (major == 3 && minor >= 8) {
            return Ok(());
        }
        return Err(SimulationEnvError::PythonVersionTooLow(ver));
    }
    Err(SimulationEnvError::PythonVersionTooLow(ver))
}

/// Resolve the simulation path relative to the executable or current working directory
pub fn get_simulation_main_py() -> Result<PathBuf, SimulationEnvError> {
    // Try multiple possible paths for both development and production modes
    let possible_paths = vec![
        // Development mode: relative to current working directory
        std::env::current_dir().map(|p| p.join("simulation/main.py")).ok(),
        // Production mode: relative to executable
        std::env::current_exe().map(|exe| {
            exe.parent().map(|parent| parent.join("../simulation/main.py"))
        }).ok().flatten(),
        // Fallback: try src-tauri relative paths
        std::env::current_dir().map(|p| p.join("src-tauri/../simulation/main.py")).ok(),
    ];

    for path in possible_paths {
        if let Some(p) = path {
            if p.exists() {
                return Ok(p);
            }
        }
    }
    
    Err(SimulationEnvError::SimulationPathNotFound)
}

/// Resolve the requirements.txt path
pub fn get_simulation_requirements() -> Result<PathBuf, SimulationEnvError> {
    // Try multiple possible paths for both development and production modes
    let possible_paths = vec![
        // Development mode: relative to current working directory
        std::env::current_dir().map(|p| p.join("simulation/requirements.txt")).ok(),
        // Production mode: relative to executable
        std::env::current_exe().map(|exe| {
            exe.parent().map(|parent| parent.join("../simulation/requirements.txt"))
        }).ok().flatten(),
        // Fallback: try src-tauri relative paths
        std::env::current_dir().map(|p| p.join("src-tauri/../simulation/requirements.txt")).ok(),
    ];

    for path in possible_paths {
        if let Some(p) = path {
            if p.exists() {
                return Ok(p);
            }
        }
    }
    
    Err(SimulationEnvError::SimulationPathNotFound)
}

/// Check if a required python package is installed (e.g. requests)
pub fn check_python_package(python_cmd: &str, package: &str) -> bool {
    let code = format!("import {}; print('ok')", package);
    if let Ok(output) = Command::new(python_cmd)
        .arg("-c").arg(&code)
        .output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.contains("ok");
    }
    false
}

/// Install python dependencies using pip
pub fn install_python_requirements(python_cmd: &str, requirements_path: &PathBuf) -> Result<(), SimulationEnvError> {
    let output = Command::new(python_cmd)
        .args(["-m", "pip", "install", "-r"])
        .arg(requirements_path)
        .output()
        .map_err(|e| SimulationEnvError::PipInstallFailed(e.to_string()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(SimulationEnvError::PipInstallFailed(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

/// Ensure simulation environment is ready (python, version, dependencies)
pub fn ensure_simulation_env() -> Result<(String, PathBuf), SimulationEnvError> {
    let python_cmd = detect_python_cmd()?;
    check_python_version(&python_cmd)?;
    let sim_main = get_simulation_main_py()?;
    let req_path = get_simulation_requirements()?;
    // Check for a common package (requests)
    if !check_python_package(&python_cmd, "requests") {
        install_python_requirements(&python_cmd, &req_path)?;
        // Re-check
        if !check_python_package(&python_cmd, "requests") {
            return Err(SimulationEnvError::DependencyCheckFailed("requests".to_string()));
        }
    }
    Ok((python_cmd, sim_main))
}