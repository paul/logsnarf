# frozen_string_literal: true

require_relative "../system/boot"
require "logsnarf"

require "benchmark/ips"
require "syslog/parser"

data = File.read("samples/1574706480345619.log")
parser_data = data.each_line.map { |l| l.split(" ", 2).last }.join

# parser = Syslog::Parser::InternalParser.new(allow_missing_structured_data: true)
parser = Syslog::Parser.new(allow_missing_structured_data: true)

Benchmark.ips do |x|
  x.report("logsnarf") do
    ms = []
    Logsnarf::Parser.new(data).each_metric { |m| ms << m }
  end

  x.report("syslog-parser") do
    ms = []
    parser_data.each_line { |l| ms << parser.parse(l) }
  end

  x.compare!
end
