#!/usr/bin/env ruby

require 'webrick'
require 'fileutils'
require 'socket'

port = ARGV[0] ? ARGV[0].to_i : 8080
docroot = File.expand_path('.')

# Function to kill existing process on port
def kill_port(port)
  begin
    output = `lsof -ti:#{port} 2>/dev/null`
    if !output.empty?
      pids = output.strip.split("\n")
      pids.each do |pid|
        puts "🔄 Killing existing process #{pid} on port #{port}..."
        `kill -9 #{pid} 2>/dev/null`
      end
      sleep 1 # Give it a moment to clean up
    end
  rescue => e
    # Ignore errors - port might just be free
  end
end

# Kill any existing process on the port
kill_port(port)

puts "🚀 Starting pitch-toy development server..."
puts "📁 Serving: #{docroot}"
puts "🌐 URL: http://localhost:#{port}/web/"
puts "📝 WASM Demo: http://localhost:#{port}/web/index.html"
puts "⏹️  Press Ctrl+C to stop\n\n"

server = WEBrick::HTTPServer.new(
  Port: port,
  DocumentRoot: docroot,
  Logger: WEBrick::Log.new(nil, WEBrick::Log::ERROR),
  AccessLog: []
)

# Add MIME type for WASM files
WEBrick::HTTPUtils::DefaultMimeTypes['wasm'] = 'application/wasm'

# Add proper headers for WASM and ES modules
server.mount_proc '/' do |req, res|
  # Set proper headers for WASM files
  if req.path.end_with?('.wasm')
    res['Content-Type'] = 'application/wasm'
    res['Cross-Origin-Embedder-Policy'] = 'require-corp'
    res['Cross-Origin-Opener-Policy'] = 'same-origin'
  end
  
  # Set proper headers for JS modules
  if req.path.end_with?('.js')
    res['Content-Type'] = 'application/javascript'
  end
  
  # Handle default file serving
  file_path = File.join(docroot, req.path)
  if File.directory?(file_path)
    index_file = File.join(file_path, 'index.html')
    if File.exist?(index_file)
      file_path = index_file
    end
  end
  
  if File.exist?(file_path) && File.file?(file_path)
    res.body = File.read(file_path)
    res.status = 200
  else
    res.status = 404
    res.body = "File not found: #{req.path}"
  end
end

trap('INT') { 
  puts "\n\n🛑 Shutting down server..."
  server.shutdown 
}

begin
  server.start
rescue => e
  puts "❌ Error starting server: #{e.message}"
  puts "💡 Try a different port: ruby serve.rb 3000"
  exit 1
end 