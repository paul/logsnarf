# frozen_string_literal: true

require "time"
module Logsnarf
  def self.Decoder(name, test, tag_fields, value_fields)
    klass = Class.new(Decoder) do
      name name
      test test
      tag_fields tag_fields
      value_fields value_fields
    end

    Logsnarf.const_set(name.split("_").map(&:capitalize).join, klass)
    klass
  end

  Decoder = Struct.new(:log_data) do
    extend Dry::Core::ClassAttributes

    defines :name, :test, :tag_fields, :value_fields

    def self.valid?(log_data)
      test.call(log_data)
    end

    def line
      log_data.line
    end

    def name
      self.class.name
    end

    def data
      {
        tags: tags,
        values: values,
        timestamp: (Time.iso8601(log_data.timestamp).to_f * 1_000_000).to_i
      }
    end

    def tags
      log_data.pairs.slice(*self.class.tag_fields)
    end

    def values
      log_data.pairs.slice(*self.class.value_fields)
              .transform_keys { |k| transform_key(k) }
              .each_with_object({}) { |(k, v), hsh| nk, nv = *move_unit_to_key(k, v); hsh[nk] = nv }
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

      [[key, unit.downcase].join("_"), (scalar.include?(".") ? Float(scalar) : Integer(scalar))]
    end
  end
end
