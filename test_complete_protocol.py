#!/usr/bin/env python3
"""
Complete PSS Protocol Test
Tests all PSS protocol event types according to the official specification
"""

import socket
import time

def send_udp_message(message, host='127.0.0.1', port=8888):
    """Send a UDP message to the specified host and port"""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.sendto(message.encode('utf-8'), (host, port))
        print(f"📤 Sent: {message}")
        sock.close()
        return True
    except Exception as e:
        print(f"❌ Failed to send message: {e}")
        return False

def test_complete_protocol():
    """Test all PSS protocol event types"""
    print("🚀 Starting complete PSS protocol test...")
    print("📋 This test covers all event types from the PSS v2.3 specification")
    
    # Wait a moment for any existing events to clear
    time.sleep(1)
    
    # Test 1: Connection and Match Setup
    print("\n📋 Test 1: Connection and Match Setup")
    send_udp_message("pre;FightLoaded;")
    time.sleep(1)
    
    # Test 2: Athletes Information
    print("\n📋 Test 2: Athletes Information")
    send_udp_message("at1;N. DESMOND;Nicolas DESMOND;MRN;at2;M. THIBAULT;Marcel THIBAULT;SUI;")
    time.sleep(2)
    
    # Test 3: Match Configuration
    print("\n📋 Test 3: Match Configuration")
    send_udp_message("mch;101;Round of 16;M- 80 kg;1;#0000ff;#FFFFFF;#ff0000;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;")
    time.sleep(2)
    
    # Test 4: Initial Warnings
    print("\n📋 Test 4: Initial Warnings")
    send_udp_message("wg1;0;wg2;0;")
    time.sleep(1)
    
    # Test 5: Winner Rounds (Initial)
    print("\n📋 Test 5: Winner Rounds (Initial)")
    send_udp_message("wrd;rd1;0;rd2;0;rd3;0;")
    time.sleep(1)
    
    # Test 6: Initial Scores
    print("\n📋 Test 6: Initial Scores")
    send_udp_message("s11;0;s21;0;s12;0;s22;0;s13;0;s23;0;")
    time.sleep(1)
    
    # Test 7: Current Scores
    print("\n📋 Test 7: Current Scores")
    send_udp_message("sc1;0;sc2;0;")
    time.sleep(1)
    
    # Test 8: Athlete Video Time
    print("\n📋 Test 8: Athlete Video Time")
    send_udp_message("avt;0;")
    time.sleep(1)
    
    # Test 9: Clock
    print("\n📋 Test 9: Clock")
    send_udp_message("clk;2:00;")
    time.sleep(1)
    
    # Test 10: Round
    print("\n📋 Test 10: Round")
    send_udp_message("rnd;1;")
    time.sleep(1)
    
    # Test 11: Fight Ready
    print("\n📋 Test 11: Fight Ready")
    send_udp_message("rdy;FightReady;")
    time.sleep(2)
    
    # Test 12: Match Start
    print("\n📋 Test 12: Match Start")
    send_udp_message("clk;2:00;start;")
    time.sleep(1)
    
    # Test 13: Points Events
    print("\n📋 Test 13: Points Events")
    send_udp_message("pt1;1;")  # Punch point for athlete 1
    time.sleep(1)
    send_udp_message("pt2;2;")  # Body point for athlete 2
    time.sleep(1)
    send_udp_message("pt1;3;")  # Head point for athlete 1
    time.sleep(1)
    
    # Test 14: Hit Level Events
    print("\n📋 Test 14: Hit Level Events")
    send_udp_message("hl1;75;")  # Hit level 75 for athlete 1
    time.sleep(1)
    send_udp_message("hl2;85;")  # Hit level 85 for athlete 2
    time.sleep(1)
    
    # Test 15: Updated Scores
    print("\n📋 Test 15: Updated Scores")
    send_udp_message("s11;4;s21;2;s12;0;s22;0;s13;0;s23;0;")
    time.sleep(1)
    send_udp_message("sc1;4;sc2;2;")
    time.sleep(1)
    
    # Test 16: Warnings
    print("\n📋 Test 16: Warnings")
    send_udp_message("wg1;1;wg2;0;")
    time.sleep(1)
    
    # Test 17: Injury Time
    print("\n📋 Test 17: Injury Time")
    send_udp_message("ij1;1:30;show;")
    time.sleep(1)
    send_udp_message("ij1;1:29;")
    time.sleep(1)
    send_udp_message("ij1;1:28;hide;")
    time.sleep(1)
    
    # Test 18: Challenge/IVR
    print("\n📋 Test 18: Challenge/IVR")
    send_udp_message("ch1;")  # Challenge requested by athlete 1
    time.sleep(1)
    send_udp_message("ch0;1;")  # Challenge accepted by referee
    time.sleep(1)
    send_udp_message("ch0;1;1;")  # Challenge won by athlete 1
    time.sleep(1)
    
    # Test 19: Break Time
    print("\n📋 Test 19: Break Time")
    send_udp_message("brk;0:59;")
    time.sleep(1)
    send_udp_message("brk;0:58;")
    time.sleep(1)
    send_udp_message("brk;0:00;stopEnd;")
    time.sleep(1)
    
    # Test 20: Round End and Winner Rounds
    print("\n📋 Test 20: Round End and Winner Rounds")
    send_udp_message("wrd;rd1;1;rd2;0;rd3;0;")  # Athlete 1 wins round 1
    time.sleep(1)
    
    # Test 21: Round 2
    print("\n📋 Test 21: Round 2")
    send_udp_message("rnd;2;")
    time.sleep(1)
    send_udp_message("clk;2:00;start;")
    time.sleep(1)
    
    # Test 22: More Points
    print("\n📋 Test 22: More Points")
    send_udp_message("pt2;3;")  # Head point for athlete 2
    time.sleep(1)
    send_udp_message("pt1;4;")  # Technical body point for athlete 1
    time.sleep(1)
    
    # Test 23: Updated Scores for Round 2
    print("\n📋 Test 23: Updated Scores for Round 2")
    send_udp_message("s11;4;s21;2;s12;4;s22;3;s13;0;s23;0;")
    time.sleep(1)
    send_udp_message("sc1;8;sc2;5;")
    time.sleep(1)
    
    # Test 24: Round 2 End
    print("\n📋 Test 24: Round 2 End")
    send_udp_message("wrd;rd1;1;rd2;1;rd3;0;")  # Athlete 1 wins round 2
    time.sleep(1)
    
    # Test 25: Match End
    print("\n📋 Test 25: Match End")
    send_udp_message("win;BLUE;")  # Blue (athlete 1) wins
    time.sleep(1)
    send_udp_message("wmh;N. DESMOND;2-0 PTF;")  # Winner with classification
    time.sleep(2)
    
    print("\n✅ Complete PSS protocol test completed!")
    print("📋 All event types from PSS v2.3 specification have been tested")
    print("📋 Check the frontend to see if all events were processed correctly")
    print("📋 Look for athlete names: Nicolas DESMOND vs Marcel THIBAULT")
    print("📋 Look for match details: Round of 16, M- 80 kg, Senior")
    print("📋 Look for scores and match progression")

if __name__ == "__main__":
    test_complete_protocol() 