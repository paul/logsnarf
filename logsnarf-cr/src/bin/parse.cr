require "option_parser"
require "../app"

filename = nil
tsdb_url = nil

parser = OptionParser.parse do |parser|
  parser.banner = "Logsnarf File Parser"

  parser.on "-f FILE", "--file=FILE", "Path of file to ingest" do |file|
    filename = file
  end
  parser.on "-d URL", "--tsdb-url=URL", "Connection URL of TSDB to use" do |url|
    tsdb_url = url
  end
  parser.on "-h", "--help", "Show help" do
    puts parser
    exit
  end
end

if filename && tsdb_url
  file = File.open(filename.not_nil!)
  App.new.extract(file, nil)
else
  puts parser
end

