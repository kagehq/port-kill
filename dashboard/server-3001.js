const http = require('http');
const port = 3001;

const server = http.createServer((req, res) => {
  res.writeHead(200, {'Content-Type': 'text/html'});
  res.end(`
    <html>
      <head><title>Vue Dev Server - Port ${port}</title></head>
      <body>
        <h1>Vue Dev Server</h1>
        <p>Server running on port ${port}</p>
        <p>PID: ${process.pid}</p>
        <p>Started: ${new Date().toISOString()}</p>
      </body>
    </html>
  `);
});

server.listen(port, () => {
  // Server started
});
