# frozen_string_literal: true

require "dry/core/class_attributes"
require "dry/core/descendants_tracker"

class Decoder
  extend Dry::Core::DescendantsTracker
  extend Dry::Core::ClassAttributes

  defines :name, :tag_fields, :value_fields

  attr :log_data

  def initialize(log_data = nil)
    @log_data = log_data
  end

  def decode(log_data)
    decoder = self.class.descendants.detect { |candidate| candidate.can_parse?(log_data) }
    return unless decoder

    decoder.new(log_data).extract_metric
  end

  def self.for(log_data)
    new(log_data) if can_parse?(log_data)
  end

  def extract_metric
    Metric.new(
      name: self.class.name,
      tags:,
      values:,
      timestamp:
    )
  end

  def tags
    log_data.pairs.slice(*self.class.tag_fields)
  end

  def values
    log_data
      .pairs.slice(*self.class.value_fields)
      .transform_keys { |k| transform_key(k) }
      .each_with_object({}) { |(k, v), hsh| nk, nv = *move_unit_to_key(k, v); hsh[nk] = nv }
  end

  def timestamp
    Time.iso8601(log_data.timestamp)
  end

  private

  SAMPLE_PREFIX = "sample#"
  def transform_key(key)
    key.delete_prefix(SAMPLE_PREFIX)
  end

  NUMBER = /^\d+(\.\d+)?/
  UNDERSCORE = "_"
  def move_unit_to_key(key, value)
    scanner = StringScanner.new(value)

    scalar = scanner.scan(NUMBER)
    unit = scanner.rest

    [[key, unit.downcase].delete_if(&:empty?).join(UNDERSCORE), Float(scalar)]
  end
end

class HerokyDynoMemory < Decoder
  name "heroky_dyno_memory"
  tag_fields %w[source]
  value_fields %w[sample#memory_total
                  sample#memory_rss
                  sample#memory_cache
                  sample#memory_swap
                  sample#memory_pgpgin
                  sample#memory_pgpgout
                  sample#memory_quota]

  def self.can_parse?(log_data)
    log_data.pairs.key?("dyno") && log_data.pairs.key?("sample#memory_total")
  end
end

class HerokuDynoLoad < Decoder
  name "heroku_dyno_load"
  tag_fields %w[source]
  value_fields %w[sample#load_avg_1m sample#load_avg_5m sample#load_avg_15m]

  def self.can_parse?(log_data)
    log_data.pairs.key?("dyno") && log_data.pairs.key?("sample#load_avg_1m")
  end
end

class HerokuRouter < Decoder
  name "heroku_router"
  tag_fields %w[method host dyno status protocol]
  value_fields %w[connect service bytes]

  def self.can_parse?(log_data)
    log_data.appname == "heroku" && log_data.procid == "router"
  end
end

class HerokuRedis < Decoder
  name "heroku_redis"
  tag_fields %w[addon]
  value_fields %w[sample#active-connections
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
                  sample#evicted-keys
                  sample#used_memory_over_limit ]

  def self.can_parse?(log_data)
    log_data.pairs["procid"] == "heroku-redis"
  end
end

class HerokuPostgres < Decoder
  name "heroku_postgres"
  tag_fields %w[addon source]
  value_fields %w[sample#current_transaction
                  sample#db_size
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
                  sample#tmp-disk-used
                  sample#tmp-disk-available
                  sample#memory-total
                  sample#memory-free
                  sample#memory-cached
                  sample#memory-postgres
                  sample#wal-percentage-used]

  def self.can_parse?(log_data)
    log_data.pairs["procid"] == "heroku-postgres"
  end
end
