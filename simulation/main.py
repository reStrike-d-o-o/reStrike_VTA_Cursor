#!/usr/bin/env python3
"""
Main entry point for tkStrike Hardware Simulator
Provides easy access to simulator functionality for integration with reStrikeVTA
"""
import sys
import os
import json
import time
from typing import Dict, Any, Optional

# Add the core directory to the path
sys.path.append(os.path.join(os.path.dirname(__file__), 'core'))

from tkstrike_hardware_simulator import (
    tkStrikeHardwareSimulator, 
    MatchScenario, 
    SimulatorMode
)

class SimulationManager:
    """Manages simulation operations for reStrikeVTA integration"""
    
    def __init__(self, host: str = "127.0.0.1", port: int = 8888):
        self.host = host
        self.port = port
        self.simulator = None
        self.is_running = False
        
    def start_simulator(self) -> bool:
        """Initialize and start the simulator"""
        try:
            self.simulator = tkStrikeHardwareSimulator(self.host, self.port)
            if self.simulator.connect():
                self.is_running = True
                return True
            return False
        except Exception as e:
            print(f"Failed to start simulator: {e}")
            return False
    
    def stop_simulator(self):
        """Stop and cleanup the simulator"""
        if self.simulator:
            self.simulator.disconnect()
            self.simulator = None
        self.is_running = False
    
    def load_scenario(self, scenario: str) -> bool:
        """Load a specific match scenario"""
        if not self.simulator or not self.is_running:
            return False
        
        try:
            scenario_enum = MatchScenario(scenario.lower())
            return self.simulator.load_match(scenario_enum)
        except Exception as e:
            print(f"Failed to load scenario {scenario}: {e}")
            return False
    
    def run_demo(self, scenario: str = "basic", duration: int = 30) -> bool:
        """Run a demo scenario"""
        if not self.simulator or not self.is_running:
            return False
        
        try:
            scenario_enum = MatchScenario(scenario.lower())
            self.simulator.run_demo(scenario_enum)
            return True
        except Exception as e:
            print(f"Failed to run demo {scenario}: {e}")
            return False
    
    def run_random(self, duration: int = 60) -> bool:
        """Run random events for specified duration"""
        if not self.simulator or not self.is_running:
            return False
        
        try:
            self.simulator.run_random(duration)
            return True
        except Exception as e:
            print(f"Failed to run random events: {e}")
            return False
    
    def add_point(self, athlete: int, point_type: int) -> bool:
        """Add a point for an athlete"""
        if not self.simulator or not self.is_running:
            return False
        
        try:
            return self.simulator.add_point(athlete, point_type)
        except Exception as e:
            print(f"Failed to add point: {e}")
            return False
    
    def add_warning(self, athlete: int) -> bool:
        """Add a warning for an athlete"""
        if not self.simulator or not self.is_running:
            return False
        
        try:
            return self.simulator.add_warning(athlete)
        except Exception as e:
            print(f"Failed to add warning: {e}")
            return False
    
    def start_injury_time(self, athlete: int = 0, duration: int = 60) -> bool:
        """Start injury time for an athlete"""
        if not self.simulator or not self.is_running:
            return False
        
        try:
            return self.simulator.start_injury_time(athlete, duration)
        except Exception as e:
            print(f"Failed to start injury time: {e}")
            return False
    
    def stop_injury_time(self, athlete: int = 0) -> bool:
        """Stop injury time for an athlete"""
        if not self.simulator or not self.is_running:
            return False
        
        try:
            return self.simulator.stop_injury_time(athlete)
        except Exception as e:
            print(f"Failed to stop injury time: {e}")
            return False
    
    def get_status(self) -> Dict[str, Any]:
        """Get current simulation status"""
        return {
            "is_running": self.is_running,
            "host": self.host,
            "port": self.port,
            "connected": self.simulator.connected if self.simulator else False
        }

def main():
    """Main function for command line usage"""
    import argparse
    
    parser = argparse.ArgumentParser(description="tkStrike Hardware Simulator")
    parser.add_argument("--host", default="127.0.0.1", help="Target host")
    parser.add_argument("--port", type=int, default=8888, help="Target port")
    parser.add_argument("--mode", choices=["demo", "random", "interactive"], default="demo", help="Simulation mode")
    parser.add_argument("--scenario", choices=["basic", "championship", "training"], default="basic", help="Match scenario")
    parser.add_argument("--duration", type=int, default=30, help="Duration for demo/random mode")
    
    args = parser.parse_args()
    
    # Create simulation manager
    manager = SimulationManager(args.host, args.port)
    
    if not manager.start_simulator():
        print("Failed to start simulator")
        return 1
    
    try:
        if args.mode == "demo":
            print(f"Running {args.scenario} demo for {args.duration} seconds...")
            manager.run_demo(args.scenario, args.duration)
        elif args.mode == "random":
            print(f"Running random events for {args.duration} seconds...")
            manager.run_random(args.duration)
        elif args.mode == "interactive":
            print("Starting interactive mode...")
            if manager.simulator:
                manager.simulator.interactive_mode()
    except KeyboardInterrupt:
        print("\nSimulation interrupted by user")
    finally:
        manager.stop_simulator()
        print("Simulation stopped")
    
    return 0

if __name__ == "__main__":
    exit(main()) 