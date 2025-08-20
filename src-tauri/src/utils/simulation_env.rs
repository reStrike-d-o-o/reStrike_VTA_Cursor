//! Simulation environment setup helpers
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
        // Production mode: relative to executable (bundled resources)
        std::env::current_exe().map(|exe| {
            exe.parent().map(|parent| parent.join("simulation/main.py"))
        }).ok().flatten(),
        // Alternative production path: resources directory
        std::env::current_exe().map(|exe| {
            exe.parent().map(|parent| parent.join("resources/simulation/main.py"))
        }).ok().flatten(),
        // Fallback: try src-tauri relative paths
        std::env::current_dir().map(|p| p.join("src-tauri/../simulation/main.py")).ok(),
        // Additional fallback: try from project root
        std::env::current_dir().map(|p| p.join("../simulation/main.py")).ok(),
    ];

    for path in &possible_paths {
        if let Some(path) = path {
            log::debug!("Checking simulation path: {:?}", path);
            if path.exists() {
                log::info!("Found simulation main.py at: {:?}", path);
                return Ok(path.clone());
            }
        }
    }

    // Log all attempted paths for debugging
    log::error!("Simulation main.py not found. Attempted paths:");
    for path in &possible_paths {
        if let Some(path) = path {
            log::error!("  - {:?} (exists: {})", path, path.exists());
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
        // Production mode: relative to executable (bundled resources)
        std::env::current_exe().map(|exe| {
            exe.parent().map(|parent| parent.join("simulation/requirements.txt"))
        }).ok().flatten(),
        // Alternative production path: resources directory
        std::env::current_exe().map(|exe| {
            exe.parent().map(|parent| parent.join("resources/simulation/requirements.txt"))
        }).ok().flatten(),
        // Fallback: try src-tauri relative paths
        std::env::current_dir().map(|p| p.join("src-tauri/../simulation/requirements.txt")).ok(),
        // Additional fallback: try from project root
        std::env::current_dir().map(|p| p.join("../simulation/requirements.txt")).ok(),
    ];

    for path in &possible_paths {
        if let Some(p) = path {
            log::debug!("Checking requirements path: {:?}", p);
            if p.exists() {
                log::info!("Found requirements.txt at: {:?}", p);
                return Ok(p.clone());
            }
        }
    }

    // Log all attempted paths for debugging
    log::error!("Simulation requirements.txt not found. Attempted paths:");
    for path in &possible_paths {
        if let Some(path) = path {
            log::error!("  - {:?} (exists: {})", path, path.exists());
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
    log::info!("Installing Python requirements from: {:?}", requirements_path);
    let output = Command::new(python_cmd)
        .args(["-m", "pip", "install", "-r"])
        .arg(requirements_path)
        .output()
        .map_err(|e| SimulationEnvError::PipInstallFailed(e.to_string()))?;
    if output.status.success() {
        log::info!("Successfully installed Python requirements");
        Ok(())
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr).to_string();
        log::error!("Failed to install Python requirements: {}", error_msg);
        Err(SimulationEnvError::PipInstallFailed(error_msg))
    }
}

/// Ensure simulation environment is ready (python, version, dependencies)
pub fn ensure_simulation_env() -> Result<(String, PathBuf), SimulationEnvError> {
    log::info!("Ensuring simulation environment is ready...");
    
    // Detect Python
    let python_cmd = detect_python_cmd()?;
    log::info!("Detected Python command: {}", python_cmd);
    
    // Check Python version
    check_python_version(&python_cmd)?;
    log::info!("Python version check passed");
    
    // Get simulation main.py path
    let sim_main = get_simulation_main_py()?;
    log::info!("Simulation main.py found at: {:?}", sim_main);
    
    // Get requirements.txt path
    let req_path = get_simulation_requirements()?;
    log::info!("Requirements.txt found at: {:?}", req_path);
    
    // Check for a common package (requests)
    if !check_python_package(&python_cmd, "requests") {
        log::info!("Installing missing Python dependencies...");
        install_python_requirements(&python_cmd, &req_path)?;
        // Re-check
        if !check_python_package(&python_cmd, "requests") {
            log::error!("Failed to install Python dependencies");
            return Err(SimulationEnvError::DependencyCheckFailed("requests".to_string()));
        }
    }
    
    log::info!("Simulation environment is ready");
    Ok((python_cmd, sim_main))
}