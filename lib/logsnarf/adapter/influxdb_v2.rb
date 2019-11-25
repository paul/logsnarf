# frozen_string_literal: true

require "async"
require "async/http/internet"

require "logsnarf/encoders/influxdb"

module Logsnarf
  module Adapter
    class InfluxdbV2
      class RequestError < StandardError
        attr_reader :response, :request

        def initialize(response)
          @response = response
        end

        def message
          %{Request failed: #{response.status}\n#{response.body&.read}}
        end
      end
      attr_reader :logger, :instrumenter

      def initialize(creds, logger:, instrumenter:)
        @creds = creds
        @logger, @instrumenter = logger.with(name: "influxdb_v2"), instrumenter
      end

      def write_metric(metrics)
        metrics = Array(metrics)
        @internet ||= Async::HTTP::Internet.new
        Async do
          body = Encoders::Influxdb.new(metrics).call
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
    end
  end
end
