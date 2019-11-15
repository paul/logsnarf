# frozen_string_literal: true

# require "logsnarf/version"
require "bundler/inline"

gemfile do
  source "https://rubygems.org"

  gem "influxdb"
  gem "dry-core"
  gem "awesome_print"
  gem "pry-byebug"
end

require "benchmark"

require "ap"
require "influxdb"
require "dry/core/class_attributes"

require_relative "logsnarf/parser"
require_relative "logsnarf/decoder"

module Logsnarf
  class Error < StandardError; end

  def self.parse(text)
    metrics = []
    time = Benchmark.measure do
      parser = Parser.new(text)
      parser.each_metric do |log_data|
        decoder = DECODERS.detect { |dec| dec.valid?(log_data) }&.new(log_data)
        metrics << decoder if decoder
      end
    end

    influx = InfluxDB::Client.new url: "http://localhost:8086/logsnarf", async: true, time_precision: "us"

    metrics.each do |metric|
      # ap metric.line
      # ap metric.name => metric.data
      influx.write_point(metric.name, metric.data)
    end

    puts "Parsed %d lines (%d bytes)" % [text.lines.size, text.bytesize]
    puts "Matched %d metrics" % metrics.size
    puts "  user       system     total    (  real    )"
    puts time
  end

  LogData = Struct.new(:line, :timestamp, :hostname, :appname, :procid, :msgid, :pairs, keyword_init: true)

  DECODERS = [
    Decoder("heroku_dyno_memory",
            ->(log_data) { log_data.pairs.key?("dyno") && log_data.pairs.key?("sample#memory_total") },
            %w[source],
            %w[sample#memory_total
               sample#memory_rss
               sample#memory_cache
               sample#memory_swap
               sample#memory_pgpgin
               sample#memory_pgpgout
               sample#memory_quota]),
    Decoder("heroku_dyno_load",
            ->(log_data) { log_data.pairs.key?("dyno") && log_data.pairs.key?("sample#load_avg_1m") },
            %w[source],
            %w[sample#load_avg_1m sample#load_avg_5m sample#load_avg_15m]),
    Decoder("heroku_redis",
            ->(log_data) { log_data.pairs["procid"] == "heroku-redis" },
            %w[addon],
            %w[sample#active-connections
               sample#load-avg-1m
               sample#load-avg-5m
               sample#load-avg-15m
               sample#read-iops
               sample#write-iops
               sample#memory-total
               sample#memory-free
               sample#memory-cached
               sample#memory-redis
               sample#hit-rate
               sample#evicted-keys]),
    Decoder("heroku_postgres",
            ->(log_data) { log_data.pairs["procid"] == "heroku-postgres" },
            %w[addon source],
            %w[sample#db_size
               sample#tables
               sample#active-connections
               sample#waiting-connections
               sample#index-cache-hit-rate
               sample#table-cache-hit-rate
               sample#load-avg-1m
               sample#load-avg-5m
               sample#load-avg-15m
               sample#read-iops
               sample#write-iops
               sample#memory-total
               sample#memory-free
               sample#memory-cached
               sample#memory-postgres]),
  ].freeze
end

if $0 == __FILE__
  data = if ARGV[0] && File.exist?(ARGV[0])
           File.read(ARGV[0])
         else
           DATA.read
         end

  Logsnarf.parse(data)
end

__END__
377 <45>1 2018-12-28T23:56:51.765168+00:00 d.5b4a0754-9a18-467b-a2e9-04aa07e84268 heroku worker.1 - - source=worker.1 dyno=heroku.97268060.72e58892-0ea8-47af-9938-476288825e83 sample#memory_total=319.86MB sample#memory_rss=254.45MB sample#memory_cache=65.41MB sample#memory_swap=0.00MB sample#memory_pgpgin=111496pages sample#memory_pgpgout=29612pages sample#memory_quota=512.00MB
253 <45>1 2018-12-18T21:10:16.557551+00:00 d.475fd4b7-03da-4e45-8c89-5d8ac5fff61d heroku sqs_worker.1 - - source=sqs_worker.1 dyno=heroku.97268060.fdbcc00f-e071-4c93-9de2-2e1ed2c2c24f sample#load_avg_1m=0.00 sample#load_avg_5m=0.00 sample#load_avg_15m=0.00
