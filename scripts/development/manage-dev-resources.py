#!/usr/bin/env python3
"""
reStrike VTA Development Resources Manager

This script manages the development environment configuration,
including port status, service status, and resource monitoring.
"""

import json
import subprocess
import sys
import os
import time
import socket
import psutil
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Any

# Colors for output
class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    PURPLE = '\033[0;35m'
    CYAN = '\033[0;36m'
    WHITE = '\033[1;37m'
    NC = '\033[0m'  # No Color

class DevResourcesManager:
    def __init__(self, config_file: str = "config/dev_resources.json"):
        self.config_file = Path(config_file)
        self.config = self.load_config()
        
    def load_config(self) -> Dict[str, Any]:
        """Load the development resources configuration."""
        try:
            with open(self.config_file, 'r') as f:
                return json.load(f)
        except FileNotFoundError:
            print(f"{Colors.RED}Error: Configuration file {self.config_file} not found.{Colors.NC}")
            sys.exit(1)
        except json.JSONDecodeError as e:
            print(f"{Colors.RED}Error: Invalid JSON in configuration file: {e}{Colors.NC}")
            sys.exit(1)
    
    def save_config(self) -> None:
        """Save the current configuration to file."""
        try:
            # Update last_updated timestamp
            self.config['development_environment']['last_updated'] = datetime.now().isoformat()
            
            with open(self.config_file, 'w') as f:
                json.dump(self.config, f, indent=2)
            print(f"{Colors.GREEN}Configuration saved successfully.{Colors.NC}")
        except Exception as e:
            print(f"{Colors.RED}Error saving configuration: {e}{Colors.NC}")
    
    def check_port_status(self, port: int) -> bool:
        """Check if a port is in use."""
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                s.settimeout(1)
                result = s.connect_ex(('localhost', port))
                return result == 0
        except Exception:
            return False
    
    def check_process_status(self, process_pattern: str) -> bool:
        """Check if a process is running."""
        try:
            for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
                try:
                    cmdline = ' '.join(proc.info['cmdline']) if proc.info['cmdline'] else ''
                    if process_pattern in cmdline:
                        return True
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    continue
            return False
        except Exception:
            return False
    
    def update_port_statuses(self) -> None:
        """Update the status of all ports in the configuration."""
        print(f"{Colors.BLUE}Checking port statuses...{Colors.NC}")
        
        for port_name, port_config in self.config['ports'].items():
            port_num = port_config['port']
            is_in_use = self.check_port_status(port_num)
            
            if is_in_use:
                port_config['status'] = 'in_use'
                print(f"  {Colors.YELLOW}⚠️  Port {port_num} ({port_config['service']}): IN USE{Colors.NC}")
            else:
                port_config['status'] = 'available'
                print(f"  {Colors.GREEN}✅ Port {port_num} ({port_config['service']}): AVAILABLE{Colors.NC}")
    
    def update_service_statuses(self) -> None:
        """Update the status of all services in the configuration."""
        print(f"{Colors.BLUE}Checking service statuses...{Colors.NC}")
        
        for service_name, service_config in self.config['services'].items():
            if service_name == 'obs_studio':
                # OBS Studio is external, skip process check
                continue
                
            command = service_config.get('command', '')
            if command:
                is_running = self.check_process_status(command)
                
                if is_running:
                    service_config['status'] = 'running'
                    print(f"  {Colors.GREEN}✅ {service_config['name']}: RUNNING{Colors.NC}")
                else:
                    service_config['status'] = 'stopped'
                    print(f"  {Colors.RED}❌ {service_config['name']}: STOPPED{Colors.NC}")
    
    def get_port_info(self, port: int) -> Optional[Dict[str, Any]]:
        """Get information about a specific port."""
        for port_name, port_config in self.config['ports'].items():
            if port_config['port'] == port:
                return port_config
        return None
    
    def get_service_info(self, service_name: str) -> Optional[Dict[str, Any]]:
        """Get information about a specific service."""
        return self.config['services'].get(service_name)
    
    def list_ports(self) -> None:
        """List all ports and their status."""
        print(f"{Colors.CYAN}📋 Port Configuration:{Colors.NC}")
        print("-" * 60)
        
        for port_name, port_config in self.config['ports'].items():
            status_icon = "✅" if port_config['status'] == 'available' else "⚠️"
            status_color = Colors.GREEN if port_config['status'] == 'available' else Colors.YELLOW
            
            print(f"{status_icon} {port_config['service']} (Port {port_config['port']})")
            print(f"   Protocol: {port_config['protocol']}")
            print(f"   Status: {status_color}{port_config['status'].upper()}{Colors.NC}")
            print(f"   Forwarded: {'Yes' if port_config['forwarded'] else 'No'}")
            if 'url' in port_config:
                print(f"   URL: {port_config['url']}")
            print()
    
    def list_services(self) -> None:
        """List all services and their status."""
        print(f"{Colors.CYAN}🔧 Service Configuration:{Colors.NC}")
        print("-" * 60)
        
        for service_name, service_config in self.config['services'].items():
            status_icon = "✅" if service_config['status'] == 'running' else "❌"
            status_color = Colors.GREEN if service_config['status'] == 'running' else Colors.RED
            
            print(f"{status_icon} {service_config['name']}")
            print(f"   Status: {status_color}{service_config['status'].upper()}{Colors.NC}")
            if 'command' in service_config:
                print(f"   Command: {service_config['command']}")
            if 'directory' in service_config:
                print(f"   Directory: {service_config['directory']}")
            print()
    
    def show_summary(self) -> None:
        """Show a summary of the development environment."""
        print(f"{Colors.PURPLE}🚀 reStrike VTA Development Environment Summary{Colors.NC}")
        print("=" * 60)
        
        # Environment info
        env = self.config['environment']
        print(f"Node.js: {env['node_version']}")
        print(f"Rust: {env['rust_version']}")
        print(f"Cargo: {env['cargo_version']}")
        print(f"mpv: {env['mpv_version']}")
        print(f"Environment: {env['container_type']}")
        print(f"OS: {env['os']}")
        print()
        
        # Port summary
        available_ports = sum(1 for p in self.config['ports'].values() if p['status'] == 'available')
        total_ports = len(self.config['ports'])
        print(f"Ports: {available_ports}/{total_ports} available")
        
        # Service summary
        running_services = sum(1 for s in self.config['services'].values() if s['status'] == 'running')
        total_services = len([s for s in self.config['services'].values() if s['status'] != 'external'])
        print(f"Services: {running_services}/{total_services} running")
        print()
        
        # Quick status
        print(f"{Colors.CYAN}Quick Status:{Colors.NC}")
        for port_name, port_config in self.config['ports'].items():
            if port_config['auto_start']:
                status_icon = "✅" if port_config['status'] == 'available' else "⚠️"
                print(f"  {status_icon} {port_config['service']} (Port {port_config['port']})")
    
    def run_health_check(self) -> None:
        """Run health checks for all services."""
        print(f"{Colors.BLUE}🏥 Running health checks...{Colors.NC}")
        
        health_checks = self.config['monitoring']['health_checks']
        
        for service, check_command in health_checks.items():
            try:
                result = subprocess.run(check_command, shell=True, capture_output=True, text=True, timeout=10)
                if result.returncode == 0:
                    print(f"  {Colors.GREEN}✅ {service}: HEALTHY{Colors.NC}")
                else:
                    print(f"  {Colors.RED}❌ {service}: UNHEALTHY{Colors.NC}")
                    print(f"     Error: {result.stderr.strip()}")
            except subprocess.TimeoutExpired:
                print(f"  {Colors.YELLOW}⚠️  {service}: TIMEOUT{Colors.NC}")
            except Exception as e:
                print(f"  {Colors.RED}❌ {service}: ERROR - {e}{Colors.NC}")
    
    def export_status_report(self, output_file: str = "dev_status_report.json") -> None:
        """Export current status to a JSON report."""
        report = {
            "timestamp": datetime.now().isoformat(),
            "environment": self.config['environment'],
            "ports": self.config['ports'],
            "services": self.config['services'],
            "summary": {
                "available_ports": sum(1 for p in self.config['ports'].values() if p['status'] == 'available'),
                "running_services": sum(1 for s in self.config['services'].values() if s['status'] == 'running'),
                "total_ports": len(self.config['ports']),
                "total_services": len([s for s in self.config['services'].values() if s['status'] != 'external'])
            }
        }
        
        try:
            with open(output_file, 'w') as f:
                json.dump(report, f, indent=2)
            print(f"{Colors.GREEN}Status report exported to {output_file}{Colors.NC}")
        except Exception as e:
            print(f"{Colors.RED}Error exporting report: {e}{Colors.NC}")

def show_help():
    """Show help information."""
    print("reStrike VTA Development Resources Manager")
    print()
    print("Usage: python scripts/manage_dev_resources.py [COMMAND]")
    print()
    print("Commands:")
    print("  status          Show current status summary")
    print("  ports           List all ports and their status")
    print("  services        List all services and their status")
    print("  update          Update all statuses")
    print("  health          Run health checks")
    print("  export [file]   Export status report (default: dev_status_report.json)")
    print("  help            Show this help message")
    print()
    print("Examples:")
    print("  python scripts/manage_dev_resources.py status")
    print("  python scripts/manage_dev_resources.py update")
    print("  python scripts/manage_dev_resources.py export my_report.json")

def main():
    """Main function."""
    if len(sys.argv) < 2:
        show_help()
        return
    
    command = sys.argv[1].lower()
    
    try:
        manager = DevResourcesManager()
        
        if command == "status":
            manager.update_port_statuses()
            manager.update_service_statuses()
            print()
            manager.show_summary()
            
        elif command == "ports":
            manager.update_port_statuses()
            print()
            manager.list_ports()
            
        elif command == "services":
            manager.update_service_statuses()
            print()
            manager.list_services()
            
        elif command == "update":
            manager.update_port_statuses()
            manager.update_service_statuses()
            manager.save_config()
            print()
            manager.show_summary()
            
        elif command == "health":
            manager.run_health_check()
            
        elif command == "export":
            output_file = sys.argv[2] if len(sys.argv) > 2 else "dev_status_report.json"
            manager.update_port_statuses()
            manager.update_service_statuses()
            manager.export_status_report(output_file)
            
        elif command == "help":
            show_help()
            
        else:
            print(f"{Colors.RED}Unknown command: {command}{Colors.NC}")
            show_help()
            
    except KeyboardInterrupt:
        print(f"\n{Colors.YELLOW}Operation cancelled by user.{Colors.NC}")
    except Exception as e:
        print(f"{Colors.RED}Error: {e}{Colors.NC}")
        sys.exit(1)

if __name__ == "__main__":
    main() 