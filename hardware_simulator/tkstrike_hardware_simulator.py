#!/usr/bin/env python3
"""
tkStrike Hardware Simulator for tkStrikeGen3
Implements PSS v2.3 protocol specification for WT competition data collection

This simulator can:
- Send realistic match data to tkStrikeGen3
- Simulate complete match flows
- Generate random events for testing
- Provide interactive control for manual testing
- Support different match scenarios and configurations

Usage:
    python tkstrike_hardware_simulator.py [options]
    
Options:
    --host HOST          Target host (default: 127.0.0.1)
    --port PORT          Target port (default: 6000)
    --mode MODE          Mode: interactive, demo, random (default: interactive)
    --match-id ID        Custom match ID (default: auto-generated)
    --scenario SCENARIO  Predefined scenario: basic, championship, training (default: basic)
"""

import socket
import time
import json
import random
import argparse
import threading
from typing import List, Dict, Any, Optional
from dataclasses import dataclass
from enum import Enum
import uuid
from datetime import datetime, timedelta

class MatchScenario(Enum):
    BASIC = "basic"
    CHAMPIONSHIP = "championship"
    TRAINING = "training"
    CUSTOM = "custom"

class SimulatorMode(Enum):
    INTERACTIVE = "interactive"
    DEMO = "demo"
    RANDOM = "random"

@dataclass
class Athlete:
    short_name: str
    long_name: str
    country_code: str
    color: str  # "blue" or "red"

@dataclass
class MatchConfig:
    number: str
    category: str
    weight: str
    rounds: int
    colors: tuple  # (bg1, fg1, bg2, fg2)
    match_id: str
    division: str
    total_rounds: int
    round_duration: int  # seconds
    countdown_type: str
    count_up: int
    format: int

class PssEventGenerator:
    """Generates PSS protocol events according to v2.3 specification"""
    
    def __init__(self):
        self.current_round = 1
        self.current_time = "2:00"
        self.athlete1_score = 0
        self.athlete2_score = 0
        self.athlete1_warnings = 0
        self.athlete2_warnings = 0
        self.clock_running = False
        self.injury_time = None
        self.break_time = None
        
    def connection_start(self) -> str:
        """Generate connection start message"""
        return "Udp Port 6000 connected;"
    
    def connection_end(self) -> str:
        """Generate connection end message"""
        return "Udp Port 6000 disconnected;"
    
    def fight_loaded(self) -> str:
        """Generate fight loaded message"""
        return "pre;FightLoaded;"
    
    def fight_ready(self) -> str:
        """Generate fight ready message"""
        return "rdy;FightReady;"
    
    def athletes(self, athlete1: Athlete, athlete2: Athlete) -> str:
        """Generate athletes information message"""
        return f"at1;{athlete1.short_name};{athlete1.long_name};{athlete1.country_code};at2;{athlete2.short_name};{athlete2.long_name};{athlete2.country_code};"
    
    def match_config(self, config: MatchConfig) -> str:
        """Generate match configuration message"""
        bg1, fg1, bg2, fg2 = config.colors
        return f"mch;{config.number};{config.category};{config.weight};{config.rounds};{bg1};{fg1};{bg2};{fg2};{config.match_id};{config.division};{config.total_rounds};{config.round_duration};{config.countdown_type};{config.count_up};{config.format};"
    
    def warnings(self, athlete1_warnings: int, athlete2_warnings: int) -> str:
        """Generate warnings/gam-jeom message"""
        return f"wg1;{athlete1_warnings};wg2;{athlete2_warnings};"
    
    def winner_rounds(self, round1_winner: int = 0, round2_winner: int = 0, round3_winner: int = 0) -> str:
        """Generate winner rounds message"""
        return f"wrd;rd1;{round1_winner};rd2;{round2_winner};rd3;{round3_winner};"
    
    def scores(self, scores: Dict[str, int]) -> List[str]:
        """Generate scores messages for all rounds"""
        messages = []
        for round_num in [1, 2, 3]:
            messages.append(f"s1{round_num};{scores.get(f'athlete1_r{round_num}', 0)};")
            messages.append(f"s2{round_num};{scores.get(f'athlete2_r{round_num}', 0)};")
        return messages
    
    def current_scores(self, athlete1_score: int, athlete2_score: int) -> List[str]:
        """Generate current scores messages"""
        return [f"sc1;{athlete1_score};", f"sc2;{athlete2_score};"]
    
    def athlete_video_time(self, time: int = 0) -> str:
        """Generate athlete video time message"""
        return f"avt;{time};"
    
    def clock(self, time: str, action: Optional[str] = None) -> str:
        """Generate clock message"""
        if action:
            return f"clk;{time};{action};"
        return f"clk;{time};"
    
    def round(self, round_num: int) -> str:
        """Generate round message"""
        return f"rnd;{round_num};"
    
    def points(self, athlete: int, point_type: int) -> str:
        """Generate points message"""
        return f"pt{athlete};{point_type};"
    
    def hit_level(self, athlete: int, level: int) -> str:
        """Generate hit level message"""
        return f"hl{athlete};{level};"
    
    def injury(self, athlete: int, time: str, action: Optional[str] = None) -> str:
        """Generate injury message"""
        if action:
            return f"ij{athlete};{time};{action};"
        return f"ij{athlete};{time};"
    
    def challenge(self, source: int, accepted: Optional[bool] = None, won: Optional[bool] = None) -> str:
        """Generate challenge/IVR message"""
        if accepted is None:
            return f"ch{source};"
        elif won is None:
            return f"ch{source};{1 if accepted else 0};"
        else:
            return f"ch{source};{1 if accepted else 0};{1 if won else 0};"
    
    def break_time(self, time: str, action: Optional[str] = None) -> str:
        """Generate break message"""
        if action:
            return f"brk;{time};{action};"
        return f"brk;{time};"
    
    def winner(self, name: str, classification: Optional[str] = None) -> str:
        """Generate winner message"""
        if classification:
            return f"wmh;{name};{classification};"
        return f"wmh;{name};"
    
    def winner_final(self, winner: str) -> str:
        """Generate final winner message"""
        return f"win;{winner.upper()};"

class MatchScenarioGenerator:
    """Generates predefined match scenarios"""
    
    @staticmethod
    def basic_match() -> Dict[str, Any]:
        """Generate basic match scenario"""
        return {
            "athlete1": Athlete("J. SMITH", "John Smith", "USA", "blue"),
            "athlete2": Athlete("A. JONES", "Alice Jones", "GBR", "red"),
            "config": MatchConfig(
                number="101",
                category="Round of 16",
                weight="M- 80 kg",
                rounds=1,
                colors=("#0000ff", "#FFFFFF", "#ff0000", "#FFFFFF"),
                match_id=str(uuid.uuid4())[:8],
                division="Senior",
                total_rounds=3,
                round_duration=120,
                countdown_type="cntDown",
                count_up=18,
                format=1
            )
        }
    
    @staticmethod
    def championship_match() -> Dict[str, Any]:
        """Generate championship match scenario"""
        return {
            "athlete1": Athlete("N. DESMOND", "Nicolas DESMOND", "MRN", "blue"),
            "athlete2": Athlete("M. THIBAULT", "Marcel THIBAULT", "SUI", "red"),
            "config": MatchConfig(
                number="245.A",
                category="Final",
                weight="M- 80 kg",
                rounds=1,
                colors=("#0000ff", "#FFFFFF", "#ff0000", "#FFFFFF"),
                match_id=str(uuid.uuid4())[:8],
                division="Senior",
                total_rounds=3,
                round_duration=120,
                countdown_type="cntDown",
                count_up=18,
                format=1
            )
        }
    
    @staticmethod
    def training_match() -> Dict[str, Any]:
        """Generate training match scenario"""
        return {
            "athlete1": Athlete("T. TRAINER", "Tom Trainer", "KOR", "blue"),
            "athlete2": Athlete("S. STUDENT", "Sam Student", "KOR", "red"),
            "config": MatchConfig(
                number="T001",
                category="Training",
                weight="M- 80 kg",
                rounds=1,
                colors=("#00ff00", "#000000", "#ff00ff", "#FFFFFF"),
                match_id=str(uuid.uuid4())[:8],
                division="Senior",
                total_rounds=3,
                round_duration=180,
                countdown_type="cntUp",
                count_up=20,
                format=1
            )
        }

class tkStrikeHardwareSimulator:
    """Main hardware simulator class"""
    
    def __init__(self, host: str = "127.0.0.1", port: int = 6000):
        self.host = host
        self.port = port
        self.socket = None
        self.connected = False
        self.event_generator = PssEventGenerator()
        self.scenario_generator = MatchScenarioGenerator()
        self.current_match = None
        self.clock_thread = None
        self.clock_running = False
        
    def connect(self) -> bool:
        """Connect to tkStrikeGen3"""
        try:
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            self.socket.settimeout(5.0)
            self.connected = True
            print(f"🔌 Connected to {self.host}:{self.port}")
            return True
        except Exception as e:
            print(f"❌ Failed to connect: {e}")
            return False
    
    def disconnect(self):
        """Disconnect from tkStrikeGen3"""
        self.stop_clock()
        if self.socket:
            self.socket.close()
        self.connected = False
        print("🔌 Disconnected")
    
    def send_message(self, message: str) -> bool:
        """Send a message to tkStrikeGen3"""
        if not self.connected:
            print("❌ Not connected")
            return False
        
        try:
            self.socket.sendto(message.encode('utf-8'), (self.host, self.port))
            print(f"📤 Sent: {message}")
            return True
        except Exception as e:
            print(f"❌ Failed to send '{message}': {e}")
            return False
    
    def send_messages(self, messages: List[str]) -> bool:
        """Send multiple messages"""
        for message in messages:
            if not self.send_message(message):
                return False
            time.sleep(0.1)  # Small delay between messages
        return True
    
    def load_match(self, scenario: MatchScenario = MatchScenario.BASIC) -> bool:
        """Load a match with the specified scenario"""
        print(f"\n🎯 Loading {scenario.value} match...")
        
        # Get scenario data
        if scenario == MatchScenario.BASIC:
            match_data = self.scenario_generator.basic_match()
        elif scenario == MatchScenario.CHAMPIONSHIP:
            match_data = self.scenario_generator.championship_match()
        elif scenario == MatchScenario.TRAINING:
            match_data = self.scenario_generator.training_match()
        else:
            print("❌ Unknown scenario")
            return False
        
        self.current_match = match_data
        
        # Send connection start
        self.send_message(self.event_generator.connection_start())
        time.sleep(0.2)
        
        # Send match load sequence
        messages = [
            self.event_generator.fight_loaded(),
            self.event_generator.athletes(match_data["athlete1"], match_data["athlete2"]),
            self.event_generator.match_config(match_data["config"]),
            self.event_generator.warnings(0, 0),
            self.event_generator.winner_rounds(),
        ]
        
        # Add scores
        messages.extend(self.event_generator.scores({
            "athlete1_r1": 0, "athlete2_r1": 0,
            "athlete1_r2": 0, "athlete2_r2": 0,
            "athlete1_r3": 0, "athlete2_r3": 0
        }))
        
        # Add current scores and other initial data
        messages.extend(self.event_generator.current_scores(0, 0))
        messages.extend([
            self.event_generator.athlete_video_time(),
            self.event_generator.clock("2:00"),
            self.event_generator.round(1),
            self.event_generator.fight_ready()
        ])
        
        return self.send_messages(messages)
    
    def start_match(self) -> bool:
        """Start the match"""
        print("\n🏁 Starting match...")
        
        messages = [
            self.event_generator.round(1),
            self.event_generator.clock("2:00", "start")
        ]
        
        if self.send_messages(messages):
            self.start_clock()
            return True
        return False
    
    def stop_match(self) -> bool:
        """Stop the match"""
        print("\n⏹️ Stopping match...")
        self.stop_clock()
        
        messages = [
            self.event_generator.clock("0:00", "stop"),
            self.event_generator.connection_end()
        ]
        
        return self.send_messages(messages)
    
    def add_point(self, athlete: int, point_type: int) -> bool:
        """Add a point for the specified athlete"""
        if athlete not in [1, 2]:
            print("❌ Invalid athlete (must be 1 or 2)")
            return False
        
        if point_type not in [1, 2, 3, 4, 5]:
            print("❌ Invalid point type (must be 1-5)")
            return False
        
        point_names = {1: "Punch", 2: "Body", 3: "Head", 4: "Technical Body", 5: "Technical Head"}
        print(f"\n🥊 Adding {point_names[point_type]} point for Athlete {athlete}")
        
        # Update scores
        if athlete == 1:
            self.event_generator.athlete1_score += point_type
        else:
            self.event_generator.athlete2_score += point_type
        
        # Send point event
        messages = [
            self.event_generator.points(athlete, point_type),
            self.event_generator.hit_level(athlete, random.randint(50, 100))
        ]
        
        # Update current scores
        messages.extend(self.event_generator.current_scores(
            self.event_generator.athlete1_score,
            self.event_generator.athlete2_score
        ))
        
        return self.send_messages(messages)
    
    def add_warning(self, athlete: int) -> bool:
        """Add a warning for the specified athlete"""
        if athlete not in [1, 2]:
            print("❌ Invalid athlete (must be 1 or 2)")
            return False
        
        print(f"\n⚠️ Adding warning for Athlete {athlete}")
        
        # Update warnings
        if athlete == 1:
            self.event_generator.athlete1_warnings += 1
        else:
            self.event_generator.athlete2_warnings += 1
        
        # Send warning event
        return self.send_message(self.event_generator.warnings(
            self.event_generator.athlete1_warnings,
            self.event_generator.athlete2_warnings
        ))
    
    def start_injury_time(self, athlete: int = 0, duration: int = 60) -> bool:
        """Start injury time"""
        print(f"\n🩹 Starting injury time for Athlete {athlete if athlete > 0 else 'Unknown'}")
        
        messages = [
            self.event_generator.injury(athlete, f"1:{duration:02d}", "show"),
            self.event_generator.clock("0:00", "stop")
        ]
        
        return self.send_messages(messages)
    
    def stop_injury_time(self, athlete: int = 0) -> bool:
        """Stop injury time"""
        print(f"\n🩹 Stopping injury time for Athlete {athlete if athlete > 0 else 'Unknown'}")
        
        messages = [
            self.event_generator.injury(athlete, "0:00", "hide"),
            self.event_generator.clock("2:00", "start")
        ]
        
        return self.send_messages(messages)
    
    def start_break(self, duration: int = 60) -> bool:
        """Start break time"""
        print(f"\n⏸️ Starting break time ({duration}s)")
        
        messages = [
            self.event_generator.break_time(f"0:{duration:02d}"),
            self.event_generator.clock("0:00", "stop")
        ]
        
        return self.send_messages(messages)
    
    def end_break(self) -> bool:
        """End break time"""
        print("\n⏸️ Ending break time")
        
        messages = [
            self.event_generator.break_time("0:00", "stopEnd"),
            self.event_generator.clock("2:00", "start")
        ]
        
        return self.send_messages(messages)
    
    def start_clock(self):
        """Start the clock countdown"""
        if self.clock_running:
            return
        
        self.clock_running = True
        self.clock_thread = threading.Thread(target=self._clock_loop)
        self.clock_thread.daemon = True
        self.clock_thread.start()
    
    def stop_clock(self):
        """Stop the clock countdown"""
        self.clock_running = False
        if self.clock_thread:
            self.clock_thread.join(timeout=1.0)
    
    def _clock_loop(self):
        """Clock countdown loop"""
        minutes, seconds = map(int, self.event_generator.current_time.split(':'))
        total_seconds = minutes * 60 + seconds
        
        while self.clock_running and total_seconds > 0:
            time.sleep(1)
            total_seconds -= 1
            
            minutes = total_seconds // 60
            seconds = total_seconds % 60
            time_str = f"{minutes}:{seconds:02d}"
            
            self.event_generator.current_time = time_str
            self.send_message(self.event_generator.clock(time_str))
            
            if total_seconds == 0:
                print("\n⏰ Round time expired!")
                break
    
    def run_demo(self, scenario: MatchScenario = MatchScenario.BASIC):
        """Run a demo match"""
        print(f"\n🎬 Running {scenario.value} demo match...")
        
        if not self.load_match(scenario):
            return
        
        time.sleep(1)
        
        if not self.start_match():
            return
        
        # Demo match flow
        time.sleep(2)
        self.add_point(1, 3)  # Head kick for athlete 1
        time.sleep(3)
        self.add_point(2, 2)  # Body kick for athlete 2
        time.sleep(2)
        self.add_warning(1)
        time.sleep(2)
        self.add_point(1, 1)  # Punch for athlete 1
        time.sleep(3)
        self.start_injury_time(2, 30)
        time.sleep(5)
        self.stop_injury_time(2)
        time.sleep(2)
        self.add_point(2, 3)  # Head kick for athlete 2
        time.sleep(2)
        
        # End match
        self.stop_match()
    
    def run_random(self, duration: int = 60):
        """Run random events for specified duration"""
        print(f"\n🎲 Running random events for {duration} seconds...")
        
        if not self.load_match(MatchScenario.BASIC):
            return
        
        time.sleep(1)
        
        if not self.start_match():
            return
        
        start_time = time.time()
        
        while time.time() - start_time < duration:
            # Random event selection
            event_type = random.choice(['point', 'warning', 'injury', 'break'])
            
            if event_type == 'point':
                athlete = random.choice([1, 2])
                point_type = random.choice([1, 2, 3, 4, 5])
                self.add_point(athlete, point_type)
            elif event_type == 'warning':
                athlete = random.choice([1, 2])
                self.add_warning(athlete)
            elif event_type == 'injury':
                athlete = random.choice([0, 1, 2])
                self.start_injury_time(athlete, random.randint(30, 60))
                time.sleep(random.randint(3, 8))
                self.stop_injury_time(athlete)
            elif event_type == 'break':
                self.start_break(random.randint(30, 60))
                time.sleep(random.randint(3, 8))
                self.end_break()
            
            time.sleep(random.randint(2, 5))
        
        self.stop_match()
    
    def interactive_mode(self):
        """Run interactive mode"""
        print("\n🎮 Interactive Mode")
        print("Available commands:")
        print("  load [basic|championship|training] - Load match")
        print("  start - Start match")
        print("  stop - Stop match")
        print("  point <athlete> <type> - Add point (1-2, 1-5)")
        print("  warning <athlete> - Add warning (1-2)")
        print("  injury <athlete> <duration> - Start injury time")
        print("  injury-stop <athlete> - Stop injury time")
        print("  break <duration> - Start break")
        print("  break-end - End break")
        print("  clock <time> - Set clock (m:ss)")
        print("  round <num> - Set round (1-3)")
        print("  quit - Exit")
        
        while True:
            try:
                command = input("\n> ").strip().lower().split()
                if not command:
                    continue
                
                cmd = command[0]
                
                if cmd == "quit":
                    break
                elif cmd == "load":
                    scenario = MatchScenario.BASIC
                    if len(command) > 1:
                        try:
                            scenario = MatchScenario(command[1])
                        except ValueError:
                            print("❌ Invalid scenario")
                            continue
                    self.load_match(scenario)
                elif cmd == "start":
                    self.start_match()
                elif cmd == "stop":
                    self.stop_match()
                elif cmd == "point":
                    if len(command) >= 3:
                        try:
                            athlete = int(command[1])
                            point_type = int(command[2])
                            self.add_point(athlete, point_type)
                        except ValueError:
                            print("❌ Invalid parameters")
                    else:
                        print("❌ Usage: point <athlete> <type>")
                elif cmd == "warning":
                    if len(command) >= 2:
                        try:
                            athlete = int(command[1])
                            self.add_warning(athlete)
                        except ValueError:
                            print("❌ Invalid athlete")
                    else:
                        print("❌ Usage: warning <athlete>")
                elif cmd == "injury":
                    if len(command) >= 3:
                        try:
                            athlete = int(command[1])
                            duration = int(command[2])
                            self.start_injury_time(athlete, duration)
                        except ValueError:
                            print("❌ Invalid parameters")
                    else:
                        print("❌ Usage: injury <athlete> <duration>")
                elif cmd == "injury-stop":
                    if len(command) >= 2:
                        try:
                            athlete = int(command[1])
                            self.stop_injury_time(athlete)
                        except ValueError:
                            print("❌ Invalid athlete")
                    else:
                        print("❌ Usage: injury-stop <athlete>")
                elif cmd == "break":
                    if len(command) >= 2:
                        try:
                            duration = int(command[1])
                            self.start_break(duration)
                        except ValueError:
                            print("❌ Invalid duration")
                    else:
                        print("❌ Usage: break <duration>")
                elif cmd == "break-end":
                    self.end_break()
                elif cmd == "clock":
                    if len(command) >= 2:
                        time_str = command[1]
                        self.send_message(self.event_generator.clock(time_str))
                    else:
                        print("❌ Usage: clock <time>")
                elif cmd == "round":
                    if len(command) >= 2:
                        try:
                            round_num = int(command[1])
                            self.send_message(self.event_generator.round(round_num))
                        except ValueError:
                            print("❌ Invalid round number")
                    else:
                        print("❌ Usage: round <num>")
                else:
                    print("❌ Unknown command")
                    
            except KeyboardInterrupt:
                print("\n👋 Goodbye!")
                break
            except Exception as e:
                print(f"❌ Error: {e}")

def main():
    parser = argparse.ArgumentParser(description="tkStrike Hardware Simulator")
    parser.add_argument("--host", default="127.0.0.1", help="Target host")
    parser.add_argument("--port", type=int, default=6000, help="Target port")
    parser.add_argument("--mode", choices=["interactive", "demo", "random"], default="interactive", help="Simulator mode")
    parser.add_argument("--scenario", choices=["basic", "championship", "training"], default="basic", help="Match scenario")
    parser.add_argument("--duration", type=int, default=60, help="Duration for random mode (seconds)")
    
    args = parser.parse_args()
    
    print("🥋 tkStrike Hardware Simulator")
    print("=" * 40)
    print(f"Target: {args.host}:{args.port}")
    print(f"Mode: {args.mode}")
    print(f"Scenario: {args.scenario}")
    
    simulator = tkStrikeHardwareSimulator(args.host, args.port)
    
    if not simulator.connect():
        return
    
    try:
        if args.mode == "interactive":
            simulator.interactive_mode()
        elif args.mode == "demo":
            scenario = MatchScenario(args.scenario)
            simulator.run_demo(scenario)
        elif args.mode == "random":
            simulator.run_random(args.duration)
    finally:
        simulator.disconnect()

if __name__ == "__main__":
    main() 