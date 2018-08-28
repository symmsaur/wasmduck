import http.server
import socketserver

handler = http.server.SimpleHTTPRequestHandler

handler.extensions_map={
       ".html": "text/html",
       ".js": "application/x-javascript",
       ".wasm": "application/wasm",
       }
httpd = socketserver.TCPServer(("", 8000), handler)
print("Serving the stuff!")
httpd.serve_forever()