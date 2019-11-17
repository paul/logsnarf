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

      def initialize(name, payload = {}, ts = self.class.now)
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

      def to_s
        out = String.new
        out << @name
        out << "," << tags unless tags.empty?
        out << " " << fields
        out << " " << @timestamp.to_s
        out
      end

      # Converts a Time to a timestamp with the given precision.
      #
      # === Example
      #
      #  InfluxDB.convert_timestamp(Time.now, "ms")
      #  #=> 1543533308243
      def self.convert_timestamp(time, precision = "u")
        factor = TIME_PRECISION_FACTORS.fetch(precision) do
          raise ArgumentError, "invalid time precision: #{precision}"
        end

        (time.to_r * factor).to_i
      end

      # Returns the current timestamp with the given precision.
      #
      # Implementation detail: This does not create an intermediate Time
      # object with `Time.now`, but directly requests the CLOCK_REALTIME,
      # which in general is a bit faster.
      #
      # This is useful, if you want or need to shave off a few microseconds
      # from your measurement.
      #
      # === Examples
      #
      #  InfluxDB.now("ns")   #=> 1543612126401392625
      #  InfluxDB.now("u")    #=> 1543612126401392
      #  InfluxDB.now("ms")   #=> 1543612126401
      #  InfluxDB.now("s")    #=> 1543612126
      #  InfluxDB.now("m")    #=> 25726868
      #  InfluxDB.now("h")    #=> 428781
      def self.now(precision = "u")
        name, divisor = CLOCK_NAMES.fetch(precision) do
          raise ArgumentError, "invalid time precision: #{precision}"
        end

        time = Process.clock_gettime Process::CLOCK_REALTIME, name
        (time / divisor).to_i
      end

      TIME_PRECISION_FACTORS = {
        "ns" => 1e9.to_r,
        nil => 1e9.to_r,
        "u" => 1e6.to_r,
        "ms" => 1e3.to_r,
        "s" => 1.to_r,
        "m" => 1.to_r / 60,
        "h" => 1.to_r / 60 / 60,
      }.freeze
      private_constant :TIME_PRECISION_FACTORS

      CLOCK_NAMES = {
        "ns" => [:nanosecond, 1],
        nil => [:nanosecond, 1],
        "u" => [:microsecond, 1],
        "ms" => [:millisecond, 1],
        "s" => [:second, 1],
        "m" => [:second, 60.to_r],
        "h" => [:second, (60 * 60).to_r],
      }.freeze
      private_constant :CLOCK_NAMES
    end
  end
end
