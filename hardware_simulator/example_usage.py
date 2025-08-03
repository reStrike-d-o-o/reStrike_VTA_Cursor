#!/usr/bin/env python3
"""
Example usage of tkStrike Hardware Simulator
Demonstrates how to use the simulator programmatically
"""

import time
from tkstrike_hardware_simulator import (
    tkStrikeHardwareSimulator, 
    MatchScenario,
    PssEventGenerator,
    MatchScenarioGenerator
)

def example_basic_usage():
    """Example of basic simulator usage"""
    print("🥋 Example: Basic Simulator Usage")
    print("=" * 40)
    
    # Create simulator instance
    simulator = tkStrikeHardwareSimulator("127.0.0.1", 6000)
    
    # Connect to tkStrikeGen3
    if not simulator.connect():
        print("❌ Could not connect to tkStrikeGen3")
        print("   Make sure tkStrikeGen3 is running and listening on port 6000")
        return
    
    try:
        # Load a basic match
        print("📋 Loading basic match...")
        simulator.load_match(MatchScenario.BASIC)
        
        # Start the match
        print("🏁 Starting match...")
        simulator.start_match()
        
        # Simulate some events
        print("🥊 Simulating match events...")
        time.sleep(2)
        
        # Add some points
        simulator.add_point(1, 3)  # Head kick for athlete 1
        time.sleep(3)
        simulator.add_point(2, 2)  # Body kick for athlete 2
        time.sleep(2)
        
        # Add a warning
        simulator.add_warning(1)
        time.sleep(2)
        
        # Add another point
        simulator.add_point(1, 1)  # Punch for athlete 1
        time.sleep(3)
        
        # Simulate injury time
        print("🩹 Simulating injury time...")
        simulator.start_injury_time(2, 30)
        time.sleep(5)
        simulator.stop_injury_time(2)
        
        # Add final points
        time.sleep(2)
        simulator.add_point(2, 3)  # Head kick for athlete 2
        time.sleep(2)
        
        # Stop the match
        print("⏹️ Stopping match...")
        simulator.stop_match()
        
        print("✅ Basic usage example completed")
        
    except Exception as e:
        print(f"❌ Error during simulation: {e}")
    finally:
        simulator.disconnect()

def example_programmatic_events():
    """Example of generating events programmatically"""
    print("\n🥋 Example: Programmatic Event Generation")
    print("=" * 40)
    
    # Create event generator
    event_generator = PssEventGenerator()
    scenario_generator = MatchScenarioGenerator()
    
    # Get championship match data
    championship = scenario_generator.championship_match()
    
    # Generate a complete match sequence
    print("📝 Generating championship match sequence...")
    
    sequence = [
        # Connection and setup
        event_generator.connection_start(),
        event_generator.fight_loaded(),
        event_generator.athletes(championship["athlete1"], championship["athlete2"]),
        event_generator.match_config(championship["config"]),
        event_generator.warnings(0, 0),
        event_generator.winner_rounds(),
        event_generator.fight_ready(),
        
        # Match start
        event_generator.round(1),
        event_generator.clock("2:00", "start"),
        
        # Match events
        event_generator.points(1, 3),  # Head kick
        event_generator.hit_level(1, 85),
        event_generator.points(2, 2),  # Body kick
        event_generator.hit_level(2, 70),
        event_generator.warnings(1, 0),  # Warning for athlete 1
        event_generator.points(1, 1),  # Punch
        event_generator.hit_level(1, 45),
        
        # Injury time
        event_generator.injury(2, "1:30", "show"),
        event_generator.clock("0:00", "stop"),
        event_generator.injury(2, "0:00", "hide"),
        event_generator.clock("2:00", "start"),
        
        # More points
        event_generator.points(2, 3),  # Head kick
        event_generator.hit_level(2, 90),
        event_generator.points(1, 4),  # Technical body
        event_generator.hit_level(1, 80),
        
        # Match end
        event_generator.clock("0:00", "stop"),
        event_generator.connection_end()
    ]
    
    print(f"✅ Generated {len(sequence)} event messages")
    
    # Display some example messages
    print("\n📋 Example messages:")
    for i, message in enumerate(sequence[:5]):
        print(f"  {i+1}. {message}")
    print("  ...")
    for i, message in enumerate(sequence[-3:]):
        print(f"  {len(sequence)-2+i}. {message}")

def example_custom_scenario():
    """Example of creating a custom scenario"""
    print("\n🥋 Example: Custom Scenario")
    print("=" * 40)
    
    from dataclasses import dataclass
    
    @dataclass
    class CustomAthlete:
        short_name: str
        long_name: str
        country_code: str
        color: str
    
    @dataclass
    class CustomMatchConfig:
        number: str
        category: str
        weight: str
        rounds: int
        colors: tuple
        match_id: str
        division: str
        total_rounds: int
        round_duration: int
        countdown_type: str
        count_up: int
        format: int
    
    # Create custom athletes
    custom_athlete1 = CustomAthlete("M. JOHNSON", "Mike Johnson", "CAN", "blue")
    custom_athlete2 = CustomAthlete("L. GARCIA", "Lisa Garcia", "ESP", "red")
    
    # Create custom match config
    custom_config = CustomMatchConfig(
        number="CUSTOM001",
        category="Custom Match",
        weight="F- 67 kg",
        rounds=1,
        colors=("#000080", "#FFFFFF", "#800000", "#FFFFFF"),  # Dark blue and dark red
        match_id="custom123",
        division="Senior",
        total_rounds=3,
        round_duration=180,  # 3 minutes
        countdown_type="cntDown",
        count_up=20,
        format=1
    )
    
    # Create event generator
    event_generator = PssEventGenerator()
    
    # Generate custom match messages
    print("📝 Generating custom match messages...")
    
    custom_messages = [
        event_generator.connection_start(),
        event_generator.fight_loaded(),
        event_generator.athletes(custom_athlete1, custom_athlete2),
        event_generator.match_config(custom_config),
        event_generator.warnings(0, 0),
        event_generator.winner_rounds(),
        event_generator.fight_ready()
    ]
    
    print(f"✅ Generated {len(custom_messages)} custom match messages")
    print("\n📋 Custom match setup:")
    for i, message in enumerate(custom_messages):
        print(f"  {i+1}. {message}")

def example_batch_processing():
    """Example of batch processing multiple events"""
    print("\n🥋 Example: Batch Event Processing")
    print("=" * 40)
    
    simulator = tkStrikeHardwareSimulator("127.0.0.1", 6000)
    
    if not simulator.connect():
        print("❌ Could not connect to tkStrikeGen3")
        return
    
    try:
        # Load match
        simulator.load_match(MatchScenario.BASIC)
        simulator.start_match()
        
        # Create batch of events
        print("📦 Processing batch of events...")
        
        events = [
            ("point", 1, 3),      # Head kick athlete 1
            ("point", 2, 2),      # Body kick athlete 2
            ("warning", 1),       # Warning athlete 1
            ("point", 1, 1),      # Punch athlete 1
            ("point", 2, 3),      # Head kick athlete 2
            ("warning", 2),       # Warning athlete 2
            ("point", 1, 4),      # Technical body athlete 1
        ]
        
        # Process events with delays
        for event_type, *params in events:
            if event_type == "point":
                athlete, point_type = params
                simulator.add_point(athlete, point_type)
                print(f"  🥊 Added {point_type} point for athlete {athlete}")
            elif event_type == "warning":
                athlete = params[0]
                simulator.add_warning(athlete)
                print(f"  ⚠️ Added warning for athlete {athlete}")
            
            time.sleep(2)  # Delay between events
        
        # Stop match
        simulator.stop_match()
        print("✅ Batch processing completed")
        
    except Exception as e:
        print(f"❌ Error during batch processing: {e}")
    finally:
        simulator.disconnect()

def main():
    """Main example function"""
    print("🥋 tkStrike Hardware Simulator Examples")
    print("=" * 50)
    
    # Run examples
    example_basic_usage()
    example_programmatic_events()
    example_custom_scenario()
    example_batch_processing()
    
    print("\n🎉 All examples completed!")
    print("\n💡 Tips:")
    print("- Make sure tkStrikeGen3 is running before running examples")
    print("- Use interactive mode for manual testing: python tkstrike_hardware_simulator.py")
    print("- Check the README.md for detailed usage instructions")
    print("- Use test_simulator.py to validate the simulator")

if __name__ == "__main__":
    main() 