# frozen_string_literal: true

module Logsnarf::Encoders
  class Influxdb
    def initialize(metrics)
      @metrics = metrics
    end

    NL = "\n"
    def call
      @metrics.map { |metric| Encoder.new(metric).to_s }.join(NL)
    end

    class Encoder
      def initialize(metric)
        @metric = metric
      end

      EQ = "="
      def tags
        @tags ||= @metric.tags.then do |tags|
          if source = tags["source"]
            type, idx = source.split(".")
            tags.merge!(type: type, idx: idx)
          end
          tags
            .transform_keys(&:to_s)
            .transform_values(&:to_s)
        end
      end

      def fields
        @fields ||= @metric.values.then do |values|
          if idx = tags["idx"]
            values["idx"] = Integer(idx)
          end

          values
            .transform_keys(&:to_s)
            .transform_values { |v| v.is_a?(Integer) ? "#{v}i" : v.to_s }
        end
      end

      def to_s
        out = String.new
        out << escape_string(@metric.name)
        unless tags.empty?
          out << COMMA
          out << tags.map { |k, v| escape_string(k) + EQ + escape_string(v) }.join(COMMA)
        end
        out << SPACE
        out << fields.map { |k, v| escape_string(k) + EQ + v }.join(COMMA)
        out << SPACE <<
          self.class
              .convert_timestamp(@metric.timestamp)
              .to_s
        out
      end

      SPACE = " "
      ESC_SPACE = '\ '
      COMMA = ","
      ESC_COMMA = '\,'
      def escape_string(str)
        str
          .to_s
          .gsub(SPACE, ESC_SPACE)
          .gsub(COMMA, ESC_COMMA)
      end

      # Converts a Time to a timestamp with the given precision.
      #
      # === Example
      #
      #  InfluxDB.convert_timestamp(Time.now, "ms")
      #  #=> 1543533308243
      USEC = "u"
      def self.convert_timestamp(time, precision = USEC)
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
      def self.now(precision = USEC)
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
