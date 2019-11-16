# frozen_string_literal: true

require "benchmark"

require "ap"
require "influxdb"
require "dry/core/class_attributes"

require_relative "logsnarf/parser"
require_relative "logsnarf/decoder"
require_relative "logsnarf/credentials"
require_relative "logsnarf/influxdb"

module Logsnarf
  class Error < StandardError; end

  class AuthError < Error; end

  class Loader
    def load(token, io)
      creds = Logsnarf.credentials.get(token)
      raise AuthError, token if creds.nil?

      influx = ::InfluxDB::Client.new url: creds["influxdb_url"], async: true, time_precision: "us"

      text = io.read
      metrics = nil
      Influxdb.instrument("load", lines: text.lines.size, bytes: text.bytes.size) do |payload|
        payload.measure("parse") do
          metrics = parse(text)
        end

        payload[:metrics] = metrics.size

        payload.measure("write") do
          metrics.each do |metric|
            influx.write_point(metric.name, metric.data)
          end
        end
      end
    end

    def parse(text)
      metrics = []
      parser = Parser.new(text)
      parser.each_metric do |log_data|
        decoder = DECODERS.detect { |dec| dec.valid?(log_data) }&.new(log_data)
        metrics << decoder if decoder
      end
      metrics
    end
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
