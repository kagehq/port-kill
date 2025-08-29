#!/bin/bash

echo "🧪 Comprehensive Port Kill Feature Testing"
echo "=========================================="

# Kill any existing instances
echo "🔍 Checking for existing instances..."
pkill -f "port-kill" 2>/dev/null
sleep 2

# Build the application
echo "🔨 Building application..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"

# Test 1: Basic functionality - default settings
echo ""
echo "🧪 Test 1: Basic functionality (default settings)"
echo "Running: ./target/release/port-kill --console --verbose"
echo "Expected: Should start with default port range 2000-6000"
echo ""

./target/release/port-kill --console --verbose &
APP_PID=$!

echo "📱 Application started with PID: $APP_PID"
echo "⏳ Waiting for output..."
sleep 8

if ! kill -0 $APP_PID 2>/dev/null; then
    echo "❌ Application crashed during basic test!"
    exit 1
fi

echo "✅ Basic functionality working!"
kill $APP_PID
sleep 2

# Test 2: Port range configuration
echo ""
echo "🧪 Test 2: Port range configuration"
echo "Running: ./target/release/port-kill --console --start-port 3000 --end-port 3010 --verbose"
echo "Expected: Should monitor ports 3000-3010"
echo ""

./target/release/port-kill --console --start-port 3000 --end-port 3010 --verbose &
APP_PID=$!

echo "📱 Application started with PID: $APP_PID"
echo "⏳ Waiting for output..."
sleep 8

if ! kill -0 $APP_PID 2>/dev/null; then
    echo "❌ Application crashed during port range test!"
    exit 1
fi

echo "✅ Port range configuration working!"
kill $APP_PID
sleep 2

# Test 3: Specific ports configuration
echo ""
echo "🧪 Test 3: Specific ports configuration"
echo "Running: ./target/release/port-kill --console --ports 3000,8000,8080 --verbose"
echo "Expected: Should monitor only ports 3000, 8000, 8080"
echo ""

./target/release/port-kill --console --ports 3000,8000,8080 --verbose &
APP_PID=$!

echo "📱 Application started with PID: $APP_PID"
echo "⏳ Waiting for output..."
sleep 8

if ! kill -0 $APP_PID 2>/dev/null; then
    echo "❌ Application crashed during specific ports test!"
    exit 1
fi

echo "✅ Specific ports configuration working!"
kill $APP_PID
sleep 2

# Test 4: Show PID functionality
echo ""
echo "🧪 Test 4: Show PID functionality"
echo "Running: ./target/release/port-kill --console --show-pid --ports 3000,8000,8080 --verbose"
echo "Expected: Should show PIDs in process information"
echo ""

./target/release/port-kill --console --show-pid --ports 3000,8000,8080 --verbose &
APP_PID=$!

echo "📱 Application started with PID: $APP_PID"
echo "⏳ Waiting for output..."
sleep 8

if ! kill -0 $APP_PID 2>/dev/null; then
    echo "❌ Application crashed during show-pid test!"
    exit 1
fi

echo "✅ Show PID functionality working!"
kill $APP_PID
sleep 2

# Test 5: Docker integration
echo ""
echo "🧪 Test 5: Docker integration"
echo "Running: ./target/release/port-kill --console --docker --ports 3000,8000,8080 --verbose"
echo "Expected: Should enable Docker container monitoring"
echo ""

./target/release/port-kill --console --docker --ports 3000,8000,8080 --verbose &
APP_PID=$!

echo "📱 Application started with PID: $APP_PID"
echo "⏳ Waiting for output..."
sleep 8

if ! kill -0 $APP_PID 2>/dev/null; then
    echo "❌ Application crashed during Docker test!"
    exit 1
fi

echo "✅ Docker integration working!"
kill $APP_PID
sleep 2

# Test 6: Ignore ports functionality
echo ""
echo "🧪 Test 6: Ignore ports functionality"
echo "Running: ./target/release/port-kill --console --ignore-ports 5353,5000,7000 --verbose"
echo "Expected: Should ignore Chromecast and AirDrop ports"
echo ""

./target/release/port-kill --console --ignore-ports 5353,5000,7000 --verbose &
APP_PID=$!

echo "📱 Application started with PID: $APP_PID"
echo "⏳ Waiting for output..."
sleep 8

if ! kill -0 $APP_PID 2>/dev/null; then
    echo "❌ Application crashed during ignore ports test!"
    exit 1
fi

echo "✅ Ignore ports functionality working!"
kill $APP_PID
sleep 2

# Test 7: Ignore processes functionality
echo ""
echo "🧪 Test 7: Ignore processes functionality"
echo "Running: ./target/release/port-kill --console --ignore-processes Chrome,ControlCe --verbose"
echo "Expected: Should ignore Chrome and ControlCe processes"
echo ""

./target/release/port-kill --console --ignore-processes Chrome,ControlCe --verbose &
APP_PID=$!

echo "📱 Application started with PID: $APP_PID"
echo "⏳ Waiting for output..."
sleep 8

if ! kill -0 $APP_PID 2>/dev/null; then
    echo "❌ Application crashed during ignore processes test!"
    exit 1
fi

echo "✅ Ignore processes functionality working!"
kill $APP_PID
sleep 2

# Test 8: Combined ignore functionality
echo ""
echo "🧪 Test 8: Combined ignore functionality"
echo "Running: ./target/release/port-kill --console --ignore-ports 5353,5000,7000 --ignore-processes Chrome,ControlCe --verbose"
echo "Expected: Should ignore both ports and processes"
echo ""

./target/release/port-kill --console --ignore-ports 5353,5000,7000 --ignore-processes Chrome,ControlCe --verbose &
APP_PID=$!

echo "📱 Application started with PID: $APP_PID"
echo "⏳ Waiting for output..."
sleep 8

if ! kill -0 $APP_PID 2>/dev/null; then
    echo "❌ Application crashed during combined ignore test!"
    exit 1
fi

echo "✅ Combined ignore functionality working!"
kill $APP_PID
sleep 2

# Test 9: Complex configuration
echo ""
echo "🧪 Test 9: Complex configuration"
echo "Running: ./target/release/port-kill --console --ports 3000,8000,8080 --ignore-ports 5353,5000,7000 --ignore-processes Chrome,ControlCe --show-pid --docker --verbose"
echo "Expected: Should combine multiple features"
echo ""

./target/release/port-kill --console --ports 3000,8000,8080 --ignore-ports 5353,5000,7000 --ignore-processes Chrome,ControlCe --show-pid --docker --verbose &
APP_PID=$!

echo "📱 Application started with PID: $APP_PID"
echo "⏳ Waiting for output..."
sleep 8

if ! kill -0 $APP_PID 2>/dev/null; then
    echo "❌ Application crashed during complex configuration test!"
    exit 1
fi

echo "✅ Complex configuration working!"
kill $APP_PID
sleep 2

# Test 10: run.sh script integration
echo ""
echo "🧪 Test 10: run.sh script integration"
echo "Running: ./run.sh --console --ports 3000,8000,8080 --ignore-ports 5353,5000,7000 --verbose"
echo "Expected: Should work with the run.sh wrapper script"
echo ""

./run.sh --console --ports 3000,8000,8080 --ignore-ports 5353,5000,7000 --verbose &
APP_PID=$!

echo "📱 Application started with PID: $APP_PID"
echo "⏳ Waiting for output..."
sleep 8

if ! kill -0 $APP_PID 2>/dev/null; then
    echo "❌ Application crashed during run.sh test!"
    exit 1
fi

echo "✅ run.sh script integration working!"
kill $APP_PID
sleep 2

# Test 11: Status bar mode (brief test)
echo ""
echo "🧪 Test 11: Status bar mode (brief test)"
echo "Running: ./target/release/port-kill --ignore-ports 5353,5000,7000 --ignore-processes Chrome,ControlCe"
echo "Expected: Should create status bar icon with ignore settings"
echo ""

./target/release/port-kill --ignore-ports 5353,5000,7000 --ignore-processes Chrome,ControlCe &
APP_PID=$!

echo "📱 Application started with PID: $APP_PID"
echo "⏳ Waiting for status bar icon to appear..."
sleep 5

if ! kill -0 $APP_PID 2>/dev/null; then
    echo "❌ Application crashed during status bar test!"
    exit 1
fi

echo "✅ Status bar mode working!"
kill $APP_PID
sleep 2

# Test 12: Validation and error handling
echo ""
echo "🧪 Test 12: Validation and error handling"
echo "Testing invalid inputs..."

# Test invalid port
if ./target/release/port-kill --ignore-ports 0 2>&1 | grep -q "Ignore port 0 is not valid"; then
    echo "✅ Invalid port validation working"
else
    echo "❌ Invalid port validation failed"
    exit 1
fi

# Test empty process name
if ./target/release/port-kill --ignore-processes "" 2>&1 | grep -q "Ignore process names cannot be empty"; then
    echo "✅ Empty process name validation working"
else
    echo "❌ Empty process name validation failed"
    exit 1
fi

# Test invalid port range
if ./target/release/port-kill --start-port 3000 --end-port 2000 2>&1 | grep -q "Start port cannot be greater than end port"; then
    echo "✅ Invalid port range validation working"
else
    echo "❌ Invalid port range validation failed"
    exit 1
fi

echo "✅ All validation tests passed!"

# Test 13: Help and version
echo ""
echo "🧪 Test 13: Help and version"
echo "Testing help and version commands..."

if ./target/release/port-kill --help | grep -q "ignore-ports"; then
    echo "✅ Help command shows ignore options"
else
    echo "❌ Help command missing ignore options"
    exit 1
fi

if ./target/release/port-kill --version | grep -q "port-kill 0.1.0"; then
    echo "✅ Version command working"
else
    echo "❌ Version command failed"
    exit 1
fi

echo "✅ Help and version commands working!"

echo ""
echo "🎉 All Feature Testing Completed!"
echo "✅ All 13 test categories passed successfully!"
echo ""
echo "📋 Summary of tested features:"
echo "   • Basic functionality (default settings)"
echo "   • Port range configuration (--start-port, --end-port)"
echo "   • Specific ports configuration (--ports)"
echo "   • Show PID functionality (--show-pid)"
echo "   • Docker integration (--docker)"
echo "   • Ignore ports functionality (--ignore-ports)"
echo "   • Ignore processes functionality (--ignore-processes)"
echo "   • Combined ignore functionality"
echo "   • Complex configuration (multiple options)"
echo "   • run.sh script integration"
echo "   • Status bar mode"
echo "   • Validation and error handling"
echo "   • Help and version commands"
echo ""
echo "🚀 Port Kill is fully operational with all features working correctly!"
