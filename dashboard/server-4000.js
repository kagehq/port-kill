const http = require('http');
const port = 4000;

const server = http.createServer((req, res) => {
  res.writeHead(200, {'Content-Type': 'text/html'});
  res.end(`
    <html>
      <head><title>Express API - Port ${port}</title></head>
      <body>
        <h1>Express API</h1>
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
