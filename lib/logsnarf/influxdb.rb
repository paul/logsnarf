# frozen_string_literal: true

require "benchmark"
require "http"

module Logsnarf
  class Influxdb
    include Singleton

    def self.instrument(name, payload = {})
      msmt = Measurement.new(name, payload)
      msmt.measure do
        yield msmt
      end

      instance.write_point(msmt)
    end

    def initialize
      @http = HTTP
              .use(logging: { logger: Logger.new(STDOUT) })
              .persistent("https://us-west-2-1.aws.cloud2.influxdata.com")
              .auth("Token tgYQYMi1g8qAfMH8Pyouq1VXMUYhE-vv_foqdc75i1w2lIgQsWx3o5DyAO4m74xddp_UyCpBqP5KJV0Nwdd5rg==")
      @params = {
        org: "paul@acceptablyunlikely.com",
        bucket: "logsnarf",
        precision: "us"
      }
    end

    def write_point(msmt)
      @http.post("/api/v2/write", params: @params, body: msmt.to_s).flush
    end

    class Measurement
      extend Forwardable

      delegate [:[], :[]=] => :@payload

      def initialize(name, payload = {}, ts = Time.now)
        @name, @payload, @timestamp = name, payload, ts
      end

      def measure(name = nil)
        out = nil
        time = Benchmark.measure do
          out = yield self
        end

        name = [name, "duration_s"].compact.join("_")

        @payload[name] = time.real
        out
      end

      def tags
        @tags ||= { env: "local" }
                  .merge(@payload.select { |_k, v| v.is_a?(String) })
                  .map { |k, v| [k, v].join("=") }
                  .join(",")
      end

      def fields
        @payload.select { |_k, v| v.is_a?(Numeric) }
                .transform_values { |v| v.is_a?(Integer) ? "#{v}i" : v.to_s }
                .map { |k, v| [k, v].join("=") }
                .join(",")
      end

      def ts
        (@timestamp.to_f * 1_000_000).to_i.to_s
      end

      def to_s
        out = String.new
        out << @name
        out << "," << tags unless tags.empty?
        out << " " << fields
        out << " " << ts
        out
      end
    end
  end
end
