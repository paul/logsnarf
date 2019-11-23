# frozen_string_literal: true

require "async"
require "async/http/internet"

module Logsnarf
  module Adapter
    class InfluxdbV2
      attr_reader :logger, :instrumenter

      def initialize(creds, logger:, instrumenter:)
        @creds = creds
        @logger, @instrumenter = logger.with(name: "influxdb_v2"), instrumenter
        @internet = Async::HTTP::Internet.new
      end

      def write_metric(metric)
        Async do
          body = encode(metric)
          response = @internet.post(url, headers, Async::HTTP::Body::Buffered.wrap(body))
          raise RequestError, response unless (200..299).cover?(response.status)
        rescue StandardError => e
          extra = {
            request: { url: url, headers: headers, body: body },
            response: response,
            creds: @creds,
            response_body: response&.read
          }
          Raven.capture_exception(e, extra: extra || {})
          raise
        end
      end

      def url
        ENV["INSTRUMENTATION_URL"]
      end

      def headers
        [["Authorization", "Token #{ENV['INSTRUMENTATION_TOKEN']}"]]
      end

      def encode(metric)
        Logsnarf::Adapter::InfluxdbV1::Measurement.new(metric).to_s
      end
    end
  end
end
