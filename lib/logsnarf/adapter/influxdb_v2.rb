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

      def url
        ENV["INSTRUMENTATION_URL"]
      end

      def headers
        [["Authorization", "Token #{ENV['INSTRUMENTATION_TOKEN']}"]]
      end

      def encode(metrics)
        metrics
          .map { |m| Logsnarf::Adapter::InfluxdbV1::Measurement.new(m) }
          .map(&:to_s)
          .join("\n")
      end
    end
  end
end
