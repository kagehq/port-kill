#!/bin/bash

# Demo script to start some development servers for testing the dashboard
# This will start several Node.js servers on different ports

echo "🚀 Starting demo development servers for Port Kill Dashboard..."

# Function to start a server on a specific port
start_server() {
    local port=$1
    local name=$2
    local file="server-${port}.js"
    
    # Create a simple Node.js server
    cat > "$file" << EOF
const http = require('http');
const port = ${port};

const server = http.createServer((req, res) => {
  res.writeHead(200, {'Content-Type': 'text/html'});
  res.end(\`
    <html>
      <head><title>${name} - Port \${port}</title></head>
      <body>
        <h1>${name}</h1>
        <p>Server running on port \${port}</p>
        <p>PID: \${process.pid}</p>
        <p>Started: \${new Date().toISOString()}</p>
      </body>
    </html>
  \`);
});

server.listen(port, () => {
  // Server started
});
EOF

    # Start the server in the background
    node "$file" &
    echo "✅ Started ${name} on port ${port} (PID: $!)"
}

# Start multiple demo servers
start_server 3000 "React Dev Server"
start_server 3001 "Vue Dev Server" 
start_server 4000 "Express API"
start_server 5000 "Next.js App"
start_server 6000 "Nuxt Dashboard"

echo ""
echo "🎉 Demo servers started!"
echo "Now open the Port Kill Dashboard at http://localhost:3001"
echo ""
echo "To stop all demo servers, run:"
echo "pkill -f 'node server-.*.js'"
echo ""
echo "Or use the dashboard to kill them individually!"
