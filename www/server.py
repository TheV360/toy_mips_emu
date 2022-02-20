import http.server
import socketserver

PORT = 8000

Handler = http.server.SimpleHTTPRequestHandler

# Correct MIME types, to make JavaScript Modules not error.
Handler.extensions_map[".html"] = "text/html"
Handler.extensions_map[".js"] = "application/javascript"
Handler.extensions_map[".jsm"] = "application/javascript"
Handler.extensions_map[".wasm"] = "application/wasm"

with socketserver.TCPServer(("", PORT), Handler) as httpd:
	print("Serving at localhost:" + str(PORT))
	
	# This will serve the page until you press Ctrl+C in the terminal.
	httpd.serve_forever()
