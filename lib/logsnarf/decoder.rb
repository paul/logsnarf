# frozen_string_literal: true

require "time"
module Logsnarf
  Metric = Struct.new(:log_data, :name, :tags, :values, :timestamp, keyword_init: true)

  class Decoder
    extend Dry::Core::ClassAttributes

    attr_reader :log_data

    defines :name, :test, :tag_fields, :value_fields

    def self.valid?(log_data)
      test.call(log_data)
    end

    def initialize(log_data)
      @log_data = log_data
    end

    def call
      Metric.new(
        log_data: log_data,
        name: self.class.name,
        tags: tags,
        values: values,
        timestamp: timestamp
      )
    end

    def tags
      log_data.pairs.slice(*self.class.tag_fields)
    end

    def values
      log_data.pairs.slice(*self.class.value_fields)
              .transform_keys { |k| transform_key(k) }
              .each_with_object({}) { |(k, v), hsh| nk, nv = *move_unit_to_key(k, v); hsh[nk] = nv }
    end

    def timestamp
      Time.iso8601(log_data.timestamp)
    end

    private

    def transform_key(key)
      key.gsub("sample#", "")
    end

    NUMBER = /^\d+(\.\d+)?/.freeze
    def move_unit_to_key(key, value)
      scanner = StringScanner.new(value)

      scalar = scanner.scan(NUMBER)
      unit = scanner.rest

      [[key, unit.downcase].delete_if(&:empty?).join("_"), Float(scalar)]
    end
  end
end
