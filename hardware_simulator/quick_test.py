#!/usr/bin/env python3
"""
Quick test script for immediate testing with reStrikeVTA
Sends a few events to verify the integration is working
"""
import time
from tkstrike_hardware_simulator import tkStrikeHardwareSimulator, MatchScenario

def quick_test():
    """Run a quick test with reStrikeVTA"""
    print("🚀 Quick Test: tkStrikeHardwareSimulator with reStrikeVTA")
    print("=" * 60)
    
    # Create simulator instance
    simulator = tkStrikeHardwareSimulator("127.0.0.1", 8888)
    
    try:
        # Connect to reStrikeVTA
        print("🔌 Connecting to reStrikeVTA (port 8888)...")
        if not simulator.connect():
            print("❌ Failed to connect to reStrikeVTA")
            print("   Make sure reStrikeVTA is running: cd src-tauri && cargo tauri dev")
            return False
        
        print("✅ Connected successfully!")
        
        # Load basic match
        print("🏆 Loading basic match scenario...")
        if not simulator.load_match(MatchScenario.BASIC):
            print("❌ Failed to load match")
            return False
        
        print("✅ Match loaded: John Smith (USA) vs Alice Jones (GBR)")
        
        # Send test events
        print("\n📡 Sending test events...")
        
        # Event 1: Blue athlete gets a punch point
        print("   🥊 Blue athlete (John Smith) gets a punch point")
        simulator.add_point(1, 1)  # Blue, punch
        time.sleep(1)
        
        # Event 2: Red athlete gets a head kick
        print("   🦵 Red athlete (Alice Jones) gets a head kick")
        simulator.add_point(2, 3)  # Red, head kick
        time.sleep(1)
        
        # Event 3: Blue athlete gets a warning
        print("   ⚠️ Blue athlete (John Smith) gets a warning")
        simulator.add_warning(1)  # Blue warning
        time.sleep(1)
        
        # Event 4: Red athlete gets a technical body point
        print("   🥋 Red athlete (Alice Jones) gets a technical body point")
        simulator.add_point(2, 2)  # Red, technical body
        time.sleep(1)
        
        # Event 5: Start injury time for blue athlete
        print("   🩹 Starting injury time for blue athlete")
        simulator.start_injury_time(1, 30)  # Blue, 30 seconds
        time.sleep(2)
        
        # Event 6: Stop injury time
        print("   ✅ Stopping injury time")
        simulator.stop_injury_time(1)
        time.sleep(1)
        
        print("\n✅ All test events sent successfully!")
        print("\n📊 Check reStrikeVTA for:")
        print("   - Events appearing in the Event Table (sidebar)")
        print("   - Scoreboard overlay updates (if open)")
        print("   - Database storage (if implemented)")
        print("   - WebSocket broadcasts to connected clients")
        
        return True
        
    except Exception as e:
        print(f"❌ Test failed: {e}")
        return False
    
    finally:
        # Cleanup
        try:
            simulator.disconnect()
            print("\n🧹 Disconnected from reStrikeVTA")
        except:
            pass

if __name__ == "__main__":
    success = quick_test()
    
    if success:
        print("\n🎉 Quick test completed successfully!")
        print("   reStrikeVTA is receiving events from the simulator")
    else:
        print("\n❌ Quick test failed")
        print("   Check that reStrikeVTA is running and listening on port 8888")
    
    print("\n💡 Next steps:")
    print("   - Run interactive mode: python tkstrike_hardware_simulator.py --interactive")
    print("   - Run full demo: python tkstrike_hardware_simulator.py --demo basic")
    print("   - Run integration tests: python test_integration.py") 