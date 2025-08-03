#!/usr/bin/env python3
"""
Main entry point for tkStrike Hardware Simulator
Provides easy access to simulator functionality for integration with reStrikeVTA
"""
import sys
import os
import json
import time
from typing import Dict, Any, Optional, List

# Add the core directory to the path
sys.path.append(os.path.join(os.path.dirname(__file__), 'core'))

from tkstrike_hardware_simulator import (
    tkStrikeHardwareSimulator, 
    MatchScenario, 
    SimulatorMode
)
from automated_simulator import AutomatedSimulator
from self_test_system import SelfTestSystem

class SimulationManager:
    """Manages simulation operations for reStrikeVTA integration"""
    
    def __init__(self, host: str = "127.0.0.1", port: int = 8888):
        self.host = host
        self.port = port
        self.simulator = None
        self.automated_simulator = None
        self.self_test_system = None
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
    
    def start_automated_simulator(self) -> bool:
        """Initialize and start the automated simulator"""
        try:
            self.automated_simulator = AutomatedSimulator(self.host, self.port)
            
            # Set up callbacks for status updates
            def status_callback(message: str):
                print(f"[AUTO] {message}")
            
            def progress_callback(current: int, total: int):
                print(f"[AUTO] Progress: {current}/{total} matches completed")
            
            self.automated_simulator.set_callbacks(status_callback, progress_callback)
            
            if self.automated_simulator.start_simulator():
                self.is_running = True
                return True
            return False
        except Exception as e:
            print(f"Failed to start automated simulator: {e}")
            return False
    
    def stop_simulator(self):
        """Stop and cleanup the simulator"""
        if self.simulator:
            self.simulator.disconnect()
            self.simulator = None
        if self.automated_simulator:
            self.automated_simulator.stop_simulator()
            self.automated_simulator = None
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
    
    def run_automated_simulation(self, scenario_name: str, custom_config: Optional[Dict[str, Any]] = None) -> bool:
        """Run automated simulation with specified scenario"""
        if not self.automated_simulator or not self.is_running:
            return False
        
        try:
            return self.automated_simulator.run_automated_simulation(scenario_name, custom_config)
        except Exception as e:
            print(f"Failed to run automated simulation: {e}")
            return False
    
    def get_automated_scenarios(self) -> list:
        """Get list of available automated scenarios"""
        try:
            # Create a temporary automated simulator just to get scenarios
            temp_simulator = AutomatedSimulator()
            return temp_simulator.get_available_scenarios()
        except Exception as e:
            print(f"Failed to get automated scenarios: {e}")
            return []
    
    def start_self_test(self) -> bool:
        """Initialize and start the self-test system"""
        try:
            self.self_test_system = SelfTestSystem(self.host, self.port)
            
            # Set up callbacks for status updates
            def status_callback(message: str):
                print(f"[SELF-TEST] {message}")
            
            def progress_callback(current: int, total: int):
                print(f"[SELF-TEST] Progress: {current}/{total} tests completed")
            
            self.self_test_system.set_callbacks(status_callback, progress_callback)
            return True
        except Exception as e:
            print(f"Failed to start self-test system: {e}")
            return False
    
    def get_self_test_categories(self) -> List[str]:
        """Get available self-test categories"""
        if not self.self_test_system:
            if not self.start_self_test():
                return []
        
        try:
            return self.self_test_system.get_available_categories()
        except Exception as e:
            print(f"Failed to get self-test categories: {e}")
            return []
    
    def run_self_test(self, selected_categories: Optional[List[str]] = None) -> Dict[str, Any]:
        """Run comprehensive self-test or selective categories"""
        if not self.self_test_system:
            if not self.start_self_test():
                return {"error": "Failed to initialize self-test system"}
        
        try:
            return self.self_test_system.run_comprehensive_test(selected_categories)
        except Exception as e:
            return {"error": f"Self-test failed: {str(e)}"}
    
    def get_self_test_report(self) -> str:
        """Get markdown format self-test report"""
        if not self.self_test_system:
            return "# Self-Test Report\n\nNo test results available."
        
        try:
            return self.self_test_system.generate_markdown_report()
        except Exception as e:
            return f"# Self-Test Report\n\nError generating report: {str(e)}"
    
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
        status = {
            "is_running": self.is_running,
            "host": self.host,
            "port": self.port,
            "connected": False,
            "automated_scenarios": self.get_automated_scenarios()
        }
        
        if self.simulator:
            status["connected"] = self.simulator.connected
        elif self.automated_simulator:
            status["connected"] = self.automated_simulator.simulator.connected if self.automated_simulator.simulator else False
            status["current_scenario"] = self.automated_simulator.current_scenario.name if self.automated_simulator.current_scenario else None
        
        return status

def main():
    """Main function for command line usage"""
    import argparse
    
    parser = argparse.ArgumentParser(description="tkStrike Hardware Simulator")
    parser.add_argument("--host", default="127.0.0.1", help="Target host")
    parser.add_argument("--port", type=int, default=8888, help="Target port")
    parser.add_argument("--mode", choices=["demo", "random", "interactive", "automated", "self-test"], default="demo", help="Simulation mode")
    parser.add_argument("--scenario", choices=["basic", "championship", "training", "quick_test", "training_session", "tournament_day", "championship"], default="basic", help="Match scenario")
    parser.add_argument("--duration", type=int, default=30, help="Duration for demo/random mode")
    parser.add_argument("--list-scenarios", action="store_true", help="List available automated scenarios")
    parser.add_argument("--self-test", action="store_true", help="Run comprehensive self-test")
    parser.add_argument("--list-test-categories", action="store_true", help="List available self-test categories")
    parser.add_argument("--test-categories", nargs="+", help="Specific test categories to run (space-separated)")
    
    args = parser.parse_args()
    
    # Create simulation manager
    manager = SimulationManager(args.host, args.port)
    
    # List scenarios if requested
    if args.list_scenarios:
        print("Available Automated Scenarios:")
        print("=" * 50)
        scenarios = manager.get_automated_scenarios()
        for scenario in scenarios:
            print(f"• {scenario['display_name']}")
            print(f"  Description: {scenario['description']}")
            print(f"  Matches: {scenario['match_count']}")
            print(f"  Est. Duration: {scenario['estimated_duration']} seconds")
            print()
        return 0
    
    # List test categories if requested
    if args.list_test_categories:
        print("Available Self-Test Categories:")
        print("=" * 50)
        categories = manager.get_self_test_categories()
        for category in categories:
            print(f"• {category}")
        print()
        return 0
    
    # Run self-test if requested
    if args.self_test or args.mode == "self-test":
        if args.test_categories:
            print(f"🧪 Starting selective self-test for categories: {', '.join(args.test_categories)}")
        else:
            print("🧪 Starting comprehensive self-test...")
        print("=" * 50)
        
        result = manager.run_self_test(args.test_categories)
        
        if "error" in result:
            print(f"❌ Self-test failed: {result['error']}")
            return 1
        
        print("\n📊 Self-Test Results:")
        print("=" * 50)
        print(f"Overall Status: {result['overall_status']}")
        print(f"Total Tests: {result['summary']['total_tests']}")
        print(f"Passed: {result['summary']['passed']} ✅")
        print(f"Failed: {result['summary']['failed']} ❌")
        print(f"Warnings: {result['summary']['warnings']} ⚠️")
        print(f"Success Rate: {result['summary']['success_rate']:.1f}%")
        print(f"Duration: {result['duration']:.2f} seconds")
        
        # Generate and save markdown report
        report = manager.get_self_test_report()
        report_path = "self_test_report.md"
        with open(report_path, "w", encoding="utf-8") as f:
            f.write(report)
        
        print(f"\n📄 Detailed report saved to: {report_path}")
        print("\n" + "=" * 50)
        return 0
    
    # Start appropriate simulator
    if args.mode == "automated":
        if not manager.start_automated_simulator():
            print("Failed to start automated simulator")
            return 1
    else:
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
        elif args.mode == "automated":
            print(f"Running automated {args.scenario} simulation...")
            manager.run_automated_simulation(args.scenario)
    except KeyboardInterrupt:
        print("\nSimulation interrupted by user")
    finally:
        manager.stop_simulator()
        print("Simulation stopped")
    
    return 0

if __name__ == "__main__":
    exit(main()) 