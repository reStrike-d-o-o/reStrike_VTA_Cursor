fn main() {
    println!("cargo:rerun-if-changed=../simulation");
    println!("cargo:rerun-if-changed=../simulation/main.py");
    println!("cargo:rerun-if-changed=../simulation/requirements.txt");
    println!("cargo:rerun-if-changed=../simulation/config");
    println!("cargo:rerun-if-changed=../simulation/core");
    println!("cargo:rerun-if-changed=../simulation/docs");
    println!("cargo:rerun-if-changed=../simulation/examples");
    println!("cargo:rerun-if-changed=../simulation/tests");
    
    // Ensure simulation directory exists and is accessible
    let simulation_dir = std::path::Path::new("../simulation");
    if !simulation_dir.exists() {
        panic!("Simulation directory not found at {:?}. Please ensure the simulation files are present.", simulation_dir);
    }
    
    // Check for critical simulation files
    let critical_files = [
        "main.py",
        "requirements.txt",
        "config/config.json",
        "core/tkstrike_hardware_simulator.py",
        "core/automated_simulator.py",
        "core/self_test_system.py"
    ];
    
    for file in &critical_files {
        let file_path = simulation_dir.join(file);
        if !file_path.exists() {
            panic!("Critical simulation file not found: {:?}. Please ensure all simulation files are present.", file_path);
        }
    }
    
    println!("cargo:warning=Simulation files verified successfully");
    
    tauri_build::build()
} 