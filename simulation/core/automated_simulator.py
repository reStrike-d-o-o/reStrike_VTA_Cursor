#!/usr/bin/env python3
"""
Automated Simulation Module for tkStrike Hardware Simulator
Provides realistic match scenario generation and multi-match simulation capabilities
"""
import random
import time
import json
import threading
from typing import List, Dict, Any, Optional, Callable
from dataclasses import dataclass
from enum import Enum
from datetime import datetime, timedelta

from tkstrike_hardware_simulator import (
    tkStrikeHardwareSimulator,
    MatchScenario,
    Athlete,
    MatchConfig
)

class SimulationType(Enum):
    SINGLE_MATCH = "single_match"
    MULTI_MATCH = "multi_match"
    TOURNAMENT = "tournament"
    TRAINING_SESSION = "training_session"

@dataclass
class AutomatedScenario:
    name: str
    description: str
    match_count: int
    duration_range: tuple  # (min, max) in seconds
    event_frequency: float  # events per second
    point_probability: float  # 0.0 to 1.0
    warning_probability: float  # 0.0 to 1.0
    injury_probability: float  # 0.0 to 1.0
    break_probability: float  # 0.0 to 1.0
    challenge_probability: float  # 0.0 to 1.0

class AthleteGenerator:
    """Generates random athlete data for realistic simulation"""
    
    # Common first names for taekwondo athletes
    FIRST_NAMES = [
        "Kim", "Lee", "Park", "Choi", "Jung", "Kang", "Yoon", "Jang", "Han", "Shin",
        "John", "Michael", "David", "James", "Robert", "William", "Richard", "Joseph", "Thomas", "Christopher",
        "Maria", "Anna", "Sofia", "Elena", "Irina", "Natalia", "Olga", "Tatiana", "Yulia", "Anastasia",
        "Ahmed", "Mohammed", "Ali", "Hassan", "Omar", "Yusuf", "Ibrahim", "Khalid", "Rashid", "Tariq"
    ]
    
    # Common last names
    LAST_NAMES = [
        "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez", "Martinez",
        "Kim", "Lee", "Park", "Choi", "Jung", "Kang", "Yoon", "Jang", "Han", "Shin",
        "Ivanov", "Petrov", "Sidorov", "Smirnov", "Popov", "Sokolov", "Lebedev", "Kozlov", "Novikov", "Morozov",
        "Al-Rashid", "Al-Zahra", "Al-Mahmoud", "Al-Sayed", "Al-Nasser", "Al-Qahtani", "Al-Shehri", "Al-Ghamdi"
    ]
    
    # IOC country codes
    COUNTRIES = [
        "KOR", "USA", "GBR", "RUS", "CHN", "JPN", "IRN", "TUR", "THA", "ESP",
        "FRA", "ITA", "GER", "BRA", "ARG", "MEX", "CAN", "AUS", "NED", "BEL",
        "POL", "CZE", "HUN", "ROU", "BUL", "SRB", "CRO", "SLO", "SVK", "EST",
        "LAT", "LTU", "FIN", "SWE", "NOR", "DEN", "ISL", "IRL", "SUI", "AUT"
    ]
    
    @staticmethod
    def generate_athlete() -> Athlete:
        """Generate a random athlete"""
        first_name = random.choice(AthleteGenerator.FIRST_NAMES)
        last_name = random.choice(AthleteGenerator.LAST_NAMES)
        country = random.choice(AthleteGenerator.COUNTRIES)
        
        return Athlete(
            short_name=f"{first_name[0]}. {last_name}",
            long_name=f"{first_name} {last_name}",
            country_code=country,
            color=random.choice(["blue", "red"])
        )

class MatchConfigGenerator:
    """Generates random match configurations"""
    
    WEIGHT_CATEGORIES = [
        "M- 54 kg", "M- 58 kg", "M- 63 kg", "M- 68 kg", "M- 74 kg", "M- 80 kg", "M- 87 kg", "M+ 87 kg",
        "W- 46 kg", "W- 49 kg", "W- 53 kg", "W- 57 kg", "W- 62 kg", "W- 67 kg", "W- 73 kg", "W+ 73 kg"
    ]
    
    CATEGORIES = [
        "Round of 16", "Quarter Final", "Semi Final", "Final", "Bronze Medal", "Gold Medal",
        "Preliminary", "Repechage", "Consolation", "Exhibition"
    ]
    
    DIVISIONS = ["Senior", "Junior", "Cadet", "Poomsae", "Para"]
    
    @staticmethod
    def generate_config() -> MatchConfig:
        """Generate a random match configuration"""
        match_number = str(random.randint(100, 999))
        category = random.choice(MatchConfigGenerator.CATEGORIES)
        weight = random.choice(MatchConfigGenerator.WEIGHT_CATEGORIES)
        division = random.choice(MatchConfigGenerator.DIVISIONS)
        
        return MatchConfig(
            number=match_number,
            category=category,
            weight=weight,
            rounds=random.randint(1, 3),
            colors=("#0000ff", "#FFFFFF", "#ff0000", "#FFFFFF"),
            match_id=str(random.randint(10000, 99999)),
            division=division,
            total_rounds=3,
            round_duration=random.choice([120, 180, 240]),  # 2, 3, or 4 minutes
            countdown_type="cntDown",
            count_up=random.randint(0, 30),
            format=1
        )

class EventSequenceGenerator:
    """Generates realistic event sequences for automated simulation"""
    
    def __init__(self, scenario: AutomatedScenario):
        self.scenario = scenario
        self.current_time = 0
        # Warning tracking per athlete per round
        self.athlete1_warnings_round = 0
        self.athlete2_warnings_round = 0
        self.current_round = 1
        self.round_start_time = 0
        self.round_duration = 120  # 2 minutes per round
    
    def generate_match_events(self, match_duration: int) -> List[Dict[str, Any]]:
        """Generate a complete sequence of events for a match"""
        events = []
        
        # Add setup events
        events.extend(self._generate_setup_events())
        
        # Generate events throughout the match
        while self.current_time < match_duration:
            # Check if round should end due to warnings
            if self.athlete1_warnings_round >= 5:
                events.extend(self._generate_round_loss_events(1))
                break
            elif self.athlete2_warnings_round >= 5:
                events.extend(self._generate_round_loss_events(2))
                break
            
            # Check if round time has expired
            if self.current_time - self.round_start_time >= self.round_duration:
                events.extend(self._generate_round_change_events())
                continue
            
            # Generate random event
            event = self._generate_random_event()
            if event:
                events.append(event)
                self.current_time += random.uniform(5, 15)  # Random time between events
            else:
                self.current_time += 10  # No event, advance time
        
        # Add conclusion events
        events.extend(self._generate_conclusion_events())
        
        return events
    
    def _generate_round_loss_events(self, losing_athlete: int) -> List[Dict[str, Any]]:
        """Generate events when an athlete loses due to 5 warnings"""
        winning_athlete = 3 - losing_athlete  # 1 becomes 2, 2 becomes 1
        
        events = [
            # Stop the clock
            {"type": "clock", "time": self.current_time, "data": {"time": "0:00", "action": "stop"}},
            # Set round winner
            {"type": "winner_rounds", "time": self.current_time + 1, "data": {"round": self.current_round, "winner": winning_athlete}},
            # End the match
            {"type": "winner", "time": self.current_time + 2, "data": {"winner": winning_athlete}},
            {"type": "winner_final", "time": self.current_time + 3, "data": {"winner": winning_athlete}}
        ]
        
        return events
    
    def _generate_setup_events(self) -> List[Dict[str, Any]]:
        """Generate initial setup events"""
        return [
            {"type": "fight_loaded", "time": 0},
            {"type": "athletes", "time": 1},
            {"type": "match_config", "time": 2},
            {"type": "fight_ready", "time": 3},
            {"type": "round", "time": 4, "data": {"round": 1}},
            {"type": "clock", "time": 5, "data": {"time": "2:00", "action": "start"}}
        ]
    
    def _generate_random_event(self) -> Optional[Dict[str, Any]]:
        """Generate a random event based on scenario probabilities"""
        rand = random.random()
        cumulative_prob = 0
        
        # Check warning probability first (to respect limits)
        if (self.athlete1_warnings_round < 5 and self.athlete2_warnings_round < 5 and 
            rand < self.scenario.warning_probability):
            return self._generate_warning_event()
        cumulative_prob += self.scenario.warning_probability
        
        # Point probability
        if rand < cumulative_prob + self.scenario.point_probability:
            return self._generate_point_event()
        cumulative_prob += self.scenario.point_probability
        
        # Injury probability
        if rand < cumulative_prob + self.scenario.injury_probability:
            return self._generate_injury_event()
        cumulative_prob += self.scenario.injury_probability
        
        # Break probability
        if rand < cumulative_prob + self.scenario.break_probability:
            return self._generate_break_event()
        cumulative_prob += self.scenario.break_probability
        
        # Challenge probability
        if rand < cumulative_prob + self.scenario.challenge_probability:
            return self._generate_challenge_event()
        
        return None
    
    def _generate_point_event(self) -> Dict[str, Any]:
        """Generate a point event"""
        athlete = random.choice([1, 2])
        point_type = random.choice([1, 2, 3, 4])  # Different point types
        
        return {
            "type": "points",
            "time": self.current_time,
            "data": {
                "athlete": athlete,
                "point_type": point_type
            }
        }
    
    def _generate_warning_event(self) -> Dict[str, Any]:
        """Generate a warning event (respecting 5-warning limit)"""
        # Only give warnings to athletes who haven't reached the limit
        available_athletes = []
        if self.athlete1_warnings_round < 5:
            available_athletes.append(1)
        if self.athlete2_warnings_round < 5:
            available_athletes.append(2)
        
        if not available_athletes:
            return None  # No warnings possible
        
        athlete = random.choice(available_athletes)
        
        # Update warning count
        if athlete == 1:
            self.athlete1_warnings_round += 1
        else:
            self.athlete2_warnings_round += 1
        
        return {
            "type": "warnings",
            "time": self.current_time,
            "data": {
                "athlete": athlete
            }
        }
    
    def _generate_injury_event(self) -> Dict[str, Any]:
        """Generate an injury event"""
        athlete = random.choice([1, 2])
        duration = random.randint(30, 120)  # 30 seconds to 2 minutes
        
        return {
            "type": "injury",
            "time": self.current_time,
            "data": {
                "athlete": athlete,
                "duration": duration,
                "action": "start"
            }
        }
    
    def _generate_break_event(self) -> Dict[str, Any]:
        """Generate a break event"""
        duration = random.randint(30, 60)  # 30 seconds to 1 minute
        
        return {
            "type": "break",
            "time": self.current_time,
            "data": {
                "duration": duration,
                "action": "start"
            }
        }
    
    def _generate_challenge_event(self) -> Dict[str, Any]:
        """Generate a challenge event"""
        source = random.choice([1, 2])  # Coach challenge
        accepted = random.choice([True, False])
        won = random.choice([True, False]) if accepted else None
        
        return {
            "type": "challenge",
            "time": self.current_time,
            "data": {
                "source": source,
                "accepted": accepted,
                "won": won
            }
        }
    
    def _generate_round_change_events(self) -> List[Dict[str, Any]]:
        """Generate round change events"""
        # Reset warning counts for new round
        self.athlete1_warnings_round = 0
        self.athlete2_warnings_round = 0
        self.current_round += 1
        self.round_start_time = self.current_time
        
        return [
            {"type": "clock", "time": self.current_time, "data": {"time": "0:00", "action": "stop"}},
            {"type": "round", "time": self.current_time + 1, "data": {"round": self.current_round}},
            {"type": "clock", "time": self.current_time + 2, "data": {"time": "2:00", "action": "start"}}
        ]
    
    def _generate_conclusion_events(self) -> List[Dict[str, Any]]:
        """Generate match conclusion events"""
        winner = random.choice([1, 2])
        
        return [
            {"type": "clock", "time": self.current_time, "data": {"time": "0:00", "action": "stop"}},
            {"type": "winner", "time": self.current_time + 1, "data": {"winner": winner}},
            {"type": "winner_final", "time": self.current_time + 2, "data": {"winner": winner}}
        ]

class AutomatedSimulator:
    """Main automated simulation controller"""
    
    def __init__(self, host: str = "127.0.0.1", port: int = 8888):
        self.host = host
        self.port = port
        self.simulator = None
        self.is_running = False
        self.current_scenario = None
        self.status_callback = None
        self.progress_callback = None
        
        # Predefined scenarios
        self.scenarios = {
            "quick_test": AutomatedScenario(
                name="Quick Test",
                description="Fast single match for testing",
                match_count=1,
                duration_range=(30, 60),
                event_frequency=0.5,
                point_probability=0.3,
                warning_probability=0.1,
                injury_probability=0.05,
                break_probability=0.02,
                challenge_probability=0.03
            ),
            "training_session": AutomatedScenario(
                name="Training Session",
                description="Multiple matches for training",
                match_count=5,
                duration_range=(60, 180),
                event_frequency=0.8,
                point_probability=0.4,
                warning_probability=0.15,
                injury_probability=0.08,
                break_probability=0.05,
                challenge_probability=0.05
            ),
            "tournament_day": AutomatedScenario(
                name="Tournament Day",
                description="Full tournament simulation",
                match_count=20,
                duration_range=(120, 300),
                event_frequency=1.2,
                point_probability=0.35,
                warning_probability=0.12,
                injury_probability=0.06,
                break_probability=0.04,
                challenge_probability=0.08
            ),
            "championship": AutomatedScenario(
                name="Championship",
                description="High-intensity championship matches",
                match_count=8,
                duration_range=(180, 360),
                event_frequency=1.5,
                point_probability=0.45,
                warning_probability=0.18,
                injury_probability=0.1,
                break_probability=0.06,
                challenge_probability=0.12
            )
        }
    
    def set_callbacks(self, status_callback: Callable[[str], None], 
                     progress_callback: Callable[[int, int], None]):
        """Set callback functions for status updates and progress"""
        self.status_callback = status_callback
        self.progress_callback = progress_callback
    
    def _update_status(self, message: str):
        """Update status via callback"""
        if self.status_callback:
            self.status_callback(message)
    
    def _update_progress(self, current: int, total: int):
        """Update progress via callback"""
        if self.progress_callback:
            self.progress_callback(current, total)
    
    def start_simulator(self) -> bool:
        """Initialize and start the simulator"""
        try:
            self.simulator = tkStrikeHardwareSimulator(self.host, self.port)
            if self.simulator.connect():
                self.is_running = True
                self._update_status("Simulator connected successfully")
                return True
            else:
                self._update_status("Failed to connect simulator")
                return False
        except Exception as e:
            self._update_status(f"Failed to start simulator: {e}")
            return False
    
    def stop_simulator(self):
        """Stop and cleanup the simulator"""
        if self.simulator:
            self.simulator.disconnect()
            self.simulator = None
        self.is_running = False
        self._update_status("Simulator stopped")
    
    def run_automated_simulation(self, scenario_name: str, custom_config: Optional[Dict[str, Any]] = None) -> bool:
        """Run automated simulation with specified scenario"""
        if not self.is_running or not self.simulator:
            self._update_status("Simulator not running")
            return False
        
        if scenario_name not in self.scenarios:
            self._update_status(f"Unknown scenario: {scenario_name}")
            return False
        
        self.current_scenario = self.scenarios[scenario_name]
        
        try:
            self._update_status(f"Starting {self.current_scenario.name} simulation...")
            
            for match_num in range(self.current_scenario.match_count):
                self._update_progress(match_num + 1, self.current_scenario.match_count)
                
                # Generate random match data
                athlete1 = AthleteGenerator.generate_athlete()
                athlete2 = AthleteGenerator.generate_athlete()
                match_config = MatchConfigGenerator.generate_config()
                
                # Ensure athletes have different colors
                if athlete1.color == athlete2.color:
                    athlete2.color = "red" if athlete1.color == "blue" else "blue"
                
                # Load match
                self._update_status(f"Loading match {match_num + 1}/{self.current_scenario.match_count}")
                self._load_custom_match(athlete1, athlete2, match_config)
                
                # Generate and execute events
                duration = random.randint(*self.current_scenario.duration_range)
                self._update_status(f"Running match {match_num + 1} for {duration} seconds")
                
                events = self._generate_and_execute_events(duration)
                
                # Wait between matches
                if match_num < self.current_scenario.match_count - 1:
                    wait_time = random.randint(5, 15)
                    self._update_status(f"Waiting {wait_time} seconds before next match...")
                    time.sleep(wait_time)
            
            self._update_status("Automated simulation completed successfully")
            return True
            
        except Exception as e:
            self._update_status(f"Automated simulation failed: {e}")
            return False
    
    def _load_custom_match(self, athlete1: Athlete, athlete2: Athlete, config: MatchConfig):
        """Load a custom match with generated data"""
        # Store the current athlete and match config data for later use
        self.current_athlete1 = athlete1
        self.current_athlete2 = athlete2
        self.current_match_config = config
        
        # Create custom scenario data
        scenario_data = {
            "athlete1": {
                "short_name": athlete1.short_name,
                "long_name": athlete1.long_name,
                "country_code": athlete1.country_code,
                "color": athlete1.color
            },
            "athlete2": {
                "short_name": athlete2.short_name,
                "long_name": athlete2.long_name,
                "country_code": athlete2.country_code,
                "color": athlete2.color
            },
            "config": {
                "number": config.number,
                "category": config.category,
                "weight": config.weight,
                "rounds": config.rounds,
                "colors": config.colors,
                "match_id": config.match_id,
                "division": config.division,
                "total_rounds": config.total_rounds,
                "round_duration": config.round_duration,
                "countdown_type": config.countdown_type,
                "count_up": config.count_up,
                "format": config.format
            }
        }
        
        # Send match setup messages
        messages = [
            self.simulator.event_generator.fight_loaded(),
            self.simulator.event_generator.athletes(athlete1, athlete2),
            self.simulator.event_generator.match_config(config),
            self.simulator.event_generator.fight_ready(),
            self.simulator.event_generator.round(1),
            self.simulator.event_generator.clock("2:00", "start")
        ]
        
        self.simulator.send_messages(messages)
    
    def _generate_and_execute_events(self, duration: int) -> List[Dict[str, Any]]:
        """Generate and execute events for a match"""
        event_generator = EventSequenceGenerator(self.current_scenario)
        events = event_generator.generate_match_events(duration)
        
        start_time = time.time()
        event_index = 0
        
        while time.time() - start_time < duration and event_index < len(events):
            event = events[event_index]
            event_time = event["time"]
            
            # Wait until it's time for this event
            elapsed = time.time() - start_time
            if elapsed < event_time:
                time.sleep(event_time - elapsed)
            
            # Execute the event
            self._execute_event(event)
            event_index += 1
            
            # Small delay between events
            time.sleep(random.uniform(0.1, 0.5))
        
        return events
    
    def _execute_event(self, event: Dict[str, Any]):
        """Execute a single event"""
        event_type = event["type"]
        data = event.get("data", {})
        
        try:
            if not self.simulator:
                self._update_status(f"Cannot execute {event_type} event: simulator not initialized")
                return
                
            if event_type == "points":
                self.simulator.add_point(data["athlete"], data["point_type"])
            elif event_type == "warnings":
                self.simulator.add_warning(data["athlete"])
            elif event_type == "injury":
                if data["action"] == "start":
                    self.simulator.start_injury_time(data["athlete"], data["duration"])
                else:
                    self.simulator.stop_injury_time(data["athlete"])
            elif event_type == "break":
                if data["action"] == "start":
                    self.simulator.start_break(data["duration"])
                else:
                    self.simulator.end_break()
            elif event_type == "challenge":
                # Handle challenge events
                source = data.get("source", 1)
                accepted = data.get("accepted", True)
                won = data.get("won", True)
                self.simulator.event_generator.challenge(source, accepted, won)
            elif event_type == "round":
                self.simulator.change_round(data["round"])
            elif event_type == "clock":
                self.simulator.event_generator.clock(data["time"], data.get("action"))
            elif event_type == "winner":
                self.simulator.event_generator.winner(f"Athlete {data['winner']}")
            elif event_type == "winner_final":
                self.simulator.event_generator.winner_final(f"Athlete {data['winner']}")
            elif event_type == "winner_rounds":
                # Handle round winner events
                round_num = data.get("round", 1)
                winner = data.get("winner", 1)
                self.simulator.event_generator.winner_rounds(
                    round1_winner=winner if round_num == 1 else 0,
                    round2_winner=winner if round_num == 2 else 0,
                    round3_winner=winner if round_num == 3 else 0
                )
            elif event_type == "fight_loaded":
                self.simulator.event_generator.fight_loaded()
            elif event_type == "athletes":
                # Send athlete data using the stored athlete information
                if hasattr(self, 'current_athlete1') and hasattr(self, 'current_athlete2'):
                    self.simulator.event_generator.athletes(self.current_athlete1, self.current_athlete2)
                else:
                    # Generate random athletes if not set
                    athlete1 = AthleteGenerator.generate_athlete()
                    athlete2 = AthleteGenerator.generate_athlete()
                    self.simulator.event_generator.athletes(athlete1, athlete2)
            elif event_type == "match_config":
                # Send match config data using the stored config information
                if hasattr(self, 'current_match_config'):
                    self.simulator.event_generator.match_config(self.current_match_config)
                else:
                    # Generate random config if not set
                    config = MatchConfigGenerator.generate_config()
                    self.simulator.event_generator.match_config(config)
            elif event_type == "fight_ready":
                self.simulator.event_generator.fight_ready()
            
            self._update_status(f"Executed {event_type} event")
            
        except Exception as e:
            self._update_status(f"Failed to execute {event_type} event: {e}")
    
    def get_available_scenarios(self) -> List[Dict[str, Any]]:
        """Get list of available scenarios"""
        return [
            {
                "name": name,
                "display_name": scenario.name,
                "description": scenario.description,
                "match_count": scenario.match_count,
                "estimated_duration": scenario.match_count * sum(scenario.duration_range) / 2
            }
            for name, scenario in self.scenarios.items()
        ]
    
    def get_status(self) -> Dict[str, Any]:
        """Get current simulation status"""
        return {
            "is_running": self.is_running,
            "current_scenario": self.current_scenario.name if self.current_scenario else None,
            "host": self.host,
            "port": self.port,
            "connected": self.simulator.connected if self.simulator else False
        } 