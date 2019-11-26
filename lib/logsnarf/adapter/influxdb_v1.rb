# frozen_string_literal: true

require "async"
require "async/http/internet"

require "logsnarf/encoders/influxdb"

module Logsnarf
  module Adapter
    class InfluxdbV1
      class RequestError < StandardError
        attr_reader :response, :request

        def initialize(response)
          @response = response
        end

        def message
          %{Request failed: #{response.status}\n#{response.body&.read}}
        end
      end
      attr_reader :logger, :instrumenter, :creds

      def initialize(creds, logger:, instrumenter:)
        @creds = creds
        @logger, @instrumenter = logger.with(name: "influxdb_v1 #{creds['name']}"), instrumenter
        @uri = URI.parse(@creds.dig("credentials", "influxdb_url"))
        @internet ||= Async::HTTP::Internet.new
        at_exit { stop }
      end

      def stop
        logger.debug "Adapter stopping"
        @task.wait
        @internet.close
      end

      def write_metrics(metrics)
        metrics = Array(metrics)
        @task = Async do
          logger.debug "sending #{metrics.size} metrics"
          body = Encoders::Influxdb.new(metrics).call
          Raven.extra_context(request: { url: url, headers: headers, body: body })
          response = @internet.post(url, headers, Async::HTTP::Body::Buffered.wrap(body))
          raise RequestError, response unless (200..299).cover?(response.status)
        rescue StandardError => e
          extra = {
            response: response,
            creds: @creds,
            response_body: response&.read,
            exception: e.inspect
          }
          Raven.capture_exception(e, extra: extra)
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
          headers << ["Authorization", "Basic #{Base64.strict_encode64(@uri.userinfo)}"] if @uri.userinfo
          headers
        end
      end
    end
  end
end
