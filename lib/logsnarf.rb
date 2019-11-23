# frozen_string_literal: true

require "benchmark"

require "ap"
require "raven"
require "dry/core/class_attributes"

require_relative "logsnarf/parser"
require_relative "logsnarf/decoder"
require_relative "logsnarf/loader"
require_relative "logsnarf/adapter"
require_relative "logsnarf/instrumenter"
require_relative "logsnarf/credentials"

module Logsnarf
  class Error < StandardError; end

  class AuthError < Error; end

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
