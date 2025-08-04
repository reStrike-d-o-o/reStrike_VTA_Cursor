use std::path::PathBuf;

fn main() {
    println!("Testing simulation path resolution...");
    
    // Test current working directory
    if let Ok(current_dir) = std::env::current_dir() {
        println!("Current working directory: {}", current_dir.display());
        
        // Test simulation/main.py path
        let sim_path = current_dir.join("simulation/main.py");
        println!("Simulation path (current dir): {}", sim_path.display());
        println!("Exists: {}", sim_path.exists());
        
        // Test requirements.txt path
        let req_path = current_dir.join("simulation/requirements.txt");
        println!("Requirements path (current dir): {}", req_path.display());
        println!("Exists: {}", req_path.exists());
    }
    
    // Test executable path
    if let Ok(exe_path) = std::env::current_exe() {
        println!("Executable path: {}", exe_path.display());
        if let Some(parent) = exe_path.parent() {
            let sim_path = parent.join("../simulation/main.py");
            println!("Simulation path (exe relative): {}", sim_path.display());
            println!("Exists: {}", sim_path.exists());
        }
    }
} 