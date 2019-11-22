# frozen_string_literal: true

module Logsnarf
  module Adapter
    class InfluxdbV2
      RequestError = Class.new(StandardError)

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

        url = ENV["INSTRUMENTATION_URL"]
        headers = [["Authorization", "Token #{ENV['INSTRUMENTATION_TOKEN']}"]]

        resp = @writer.post(url.to_s, headers, body)

        raise RequestError, resp unless (200..299).cover?(resp.status)
      end
    end
  end
end
