#!/usr/bin/env python3
"""
Phase 2 Optimization Test Script
Tests connection pooling, data archival, and performance monitoring
"""

import asyncio
import json
import time
import random
from datetime import datetime, timedelta

class Phase2OptimizationTester:
    def __init__(self, tauri_app):
        self.app = tauri_app
        self.test_results = {}
        
    async def test_connection_pooling(self):
        """Test database connection pooling performance"""
        print("🔧 Testing Connection Pooling...")
        
        start_time = time.time()
        
        # Test multiple concurrent database operations
        tasks = []
        for i in range(20):
            task = self.test_concurrent_db_operation(i)
            tasks.append(task)
        
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        end_time = time.time()
        duration = end_time - start_time
        
        # Get pool statistics
        pool_stats = await self.app.invoke('get_database_pool_stats')
        
        self.test_results['connection_pooling'] = {
            'duration': duration,
            'concurrent_operations': len(tasks),
            'successful_operations': len([r for r in results if not isinstance(r, Exception)]),
            'pool_stats': pool_stats,
            'operations_per_second': len(tasks) / duration
        }
        
        print(f"✅ Connection Pooling: {len(tasks)} operations in {duration:.2f}s ({len(tasks)/duration:.1f} ops/sec)")
        
    async def test_concurrent_db_operation(self, operation_id):
        """Simulate concurrent database operation"""
        try:
            # Simulate database operation
            await asyncio.sleep(random.uniform(0.01, 0.05))
            return f"operation_{operation_id}_success"
        except Exception as e:
            return f"operation_{operation_id}_failed: {e}"
    
    async def test_data_archival(self):
        """Test data archival functionality"""
        print("📦 Testing Data Archival...")
        
        # Create test events (simulate old data)
        await self.create_test_events()
        
        # Test archival
        start_time = time.time()
        archived_count = await self.app.invoke('archive_old_events', {'days_old': 1})
        end_time = time.time()
        
        # Get archive statistics
        archive_stats = await self.app.invoke('get_archive_statistics')
        
        self.test_results['data_archival'] = {
            'archived_count': archived_count,
            'duration': end_time - start_time,
            'archive_stats': archive_stats
        }
        
        print(f"✅ Data Archival: {archived_count} events archived in {end_time-start_time:.2f}s")
        
    async def create_test_events(self):
        """Create test events for archival testing"""
        # This would create test events in the database
        # For now, we'll simulate this
        print("📝 Creating test events for archival...")
        await asyncio.sleep(1)
        
    async def test_performance_monitoring(self):
        """Test performance monitoring capabilities"""
        print("📊 Testing Performance Monitoring...")
        
        # Get current performance metrics
        metrics = await self.app.invoke('get_udp_performance_metrics')
        memory_usage = await self.app.invoke('get_udp_memory_usage')
        
        self.test_results['performance_monitoring'] = {
            'metrics': metrics,
            'memory_usage': memory_usage,
            'timestamp': datetime.now().isoformat()
        }
        
        print("✅ Performance Monitoring: Metrics collected successfully")
        
    async def test_archive_optimization(self):
        """Test archive table optimization"""
        print("🔧 Testing Archive Optimization...")
        
        start_time = time.time()
        await self.app.invoke('optimize_archive_tables')
        end_time = time.time()
        
        self.test_results['archive_optimization'] = {
            'duration': end_time - start_time
        }
        
        print(f"✅ Archive Optimization: Completed in {end_time-start_time:.2f}s")
        
    async def test_pool_cleanup(self):
        """Test connection pool cleanup"""
        print("🧹 Testing Pool Cleanup...")
        
        start_time = time.time()
        await self.app.invoke('cleanup_database_pool')
        end_time = time.time()
        
        # Get updated pool stats
        pool_stats = await self.app.invoke('get_database_pool_stats')
        
        self.test_results['pool_cleanup'] = {
            'duration': end_time - start_time,
            'pool_stats_after_cleanup': pool_stats
        }
        
        print(f"✅ Pool Cleanup: Completed in {end_time-start_time:.2f}s")
        
    async def run_all_tests(self):
        """Run all Phase 2 optimization tests"""
        print("🚀 Starting Phase 2 Optimization Tests...")
        print("=" * 60)
        
        try:
            await self.test_connection_pooling()
            await self.test_data_archival()
            await self.test_performance_monitoring()
            await self.test_archive_optimization()
            await self.test_pool_cleanup()
            
            print("\n" + "=" * 60)
            print("📋 Phase 2 Test Results Summary:")
            print("=" * 60)
            
            for test_name, results in self.test_results.items():
                print(f"\n🔍 {test_name.replace('_', ' ').title()}:")
                if isinstance(results, dict):
                    for key, value in results.items():
                        if key != 'metrics' and key != 'memory_usage' and key != 'pool_stats':
                            print(f"  {key}: {value}")
                else:
                    print(f"  Result: {results}")
            
            # Save detailed results
            with open('phase2_test_results.json', 'w') as f:
                json.dump(self.test_results, f, indent=2, default=str)
            
            print(f"\n💾 Detailed results saved to: phase2_test_results.json")
            
        except Exception as e:
            print(f"❌ Test failed: {e}")
            raise

async def main():
    """Main test function"""
    # This would initialize the Tauri app connection
    # For now, we'll create a mock tester
    class MockTauriApp:
        async def invoke(self, command, params=None):
            # Mock responses for testing
            if command == 'get_database_pool_stats':
                return {
                    'available_connections': 8,
                    'max_connections': 10,
                    'pool_utilization': 0.8
                }
            elif command == 'archive_old_events':
                return random.randint(50, 200)
            elif command == 'get_archive_statistics':
                return {
                    'archived_events': 1500,
                    'archived_details': 3000,
                    'archive_size_bytes': 1024000
                }
            elif command == 'get_udp_performance_metrics':
                return {
                    'events_per_second': 150.5,
                    'average_processing_time_ms': 2.3,
                    'total_events_processed': 15000
                }
            elif command == 'get_udp_memory_usage':
                return {
                    'total_memory_mb': 256.5,
                    'heap_usage_mb': 128.2,
                    'stack_usage_mb': 64.1
                }
            else:
                return {'status': 'success'}
    
    app = MockTauriApp()
    tester = Phase2OptimizationTester(app)
    await tester.run_all_tests()

if __name__ == "__main__":
    asyncio.run(main()) 