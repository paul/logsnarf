# frozen_string_literal: true

module Logsnarf
  module Adapter
    class InfluxdbV2
      attr_reader :logger, :instrumenter

      def initialize(creds, logger:, instrumenter:)
        @creds = creds
        @logger, @instrumenter = logger.with(name: "influxdb_v2"), instrumenter
      end

      def write_metric(metric)
        writer.push(Array(metric))
      end

      def writer
        @writer ||= Writer.new(adapter: self)
      end

      def stop
        writer.stop
      end

      def publish(metrics)
        body = metrics
               .map { |m| Logsnarf::Adapter::InfluxdbV1::Measurement.new(m) }
               .map(&:to_s)
               .join("\n")

        query = URI.encode_www_form(org: "paul@acceptablyunlikely.com", bucket: "logsnarf", precision: "us")
        url = URI::HTTP.build(host: ENV["INFLUXDB_HOST"], port: "9999", path: "/api/v2/write", query: query)
        headers = [["Authorization", "Token #{ENV['INFLUXDB_TOKEN']}"]]

        resp = @writer.post(url.to_s, headers, body)

        raise RequestError, resp unless (200..299).cover?(resp.status)
      end
    end
  end
end
