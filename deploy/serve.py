import http.server
import socketserver
import shutil
import sys

handler = http.server.SimpleHTTPRequestHandler

handler.extensions_map={
       ".html": "text/html",
       ".js": "application/x-javascript",
       ".wasm": "application/wasm",
       ".png": "image/png",
       }
shutil.copyfile("../target/wasm32-unknown-unknown/debug/wasmduck.wasm", "wasmduck.wasm")
httpd = socketserver.TCPServer(("", int(sys.argv[1])), handler)
print("Serving the stuff!")
try:
    httpd.serve_forever()
finally:
    httpd.stop()