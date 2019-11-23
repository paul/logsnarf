# frozen_string_literal: true

require "async"
require "async/http/internet"

module Logsnarf
  module Adapter
    class InfluxdbV1
      attr_reader :logger, :instrumenter, :creds

      def initialize(creds, logger:, instrumenter:)
        @creds = creds
        @logger, @instrumenter = logger.with(name: "influxdb_v1 #{creds['token']}"), instrumenter
        @uri = URI.parse(@creds.dig("credentials", "influxdb_url"))
        @internet = Async::HTTP::Internet.new
      end

      def write_metric(metric)
        Async do
          request = [url, headers, encode(metric)]
          response = @internet.post(*request)
          raise RequestError, response unless (200..299).cover?(response.status)
        rescue StandardError => e
          extra = {
            request: request,
            response: response,
            creds: @creds,
            response_body: response&.read
          }
          Raven.capture_exception(e, extra: extra || {})
          raise
        end
      end

      def url
        @url ||= begin
          query = URI.encode_www_form(
            db: @uri.path.split("/").last,
            precision: "u"
          )

          builder = (@uri.scheme == "https" ? URI::HTTPS : URI::HTTP)
          builder.build(host: @uri.host, port: @uri.port, path: "/write", query: query).to_s
        end
      end

      def headers
        @headers ||= begin
          headers = []
          headers << ["Authorization", "Basic #{Base64.encode64(@uri.userinfo)}"] if @uri.userinfo
          headers
        end
      end

      def encode(metric)
        Logsnarf::Adapter::InfluxdbV1::Measurement.new(metric).to_s
      end

    private

      class Measurement
        def initialize(metric)
          @metric = metric
        end

        def tags
          @tags ||= @metric.tags
                           .map { |k, v| [escape_string(k), escape_string(v)].join("=") }
                           .join(",")
        end

        def fields
          @metric.values
                 .transform_values { |v| v.is_a?(Integer) ? "#{v}i" : v.to_s }
                 .map { |k, v| [k, v].join("=") }
                 .join(",")
        end

        def to_s
          out = String.new
          out << escape_string(@metric.name)
          out << "," << tags unless tags.empty?
          out << " " << fields
          out << " " << self.class.convert_timestamp(@metric.timestamp).to_s
          out
        end

        def escape_string(str)
          str.to_s.gsub(" ", '\ ').gsub(",", '\,')
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
end
