# frozen_string_literal: true

require "import"

module Logsnarf::Clients
  class InfluxdbV1
    include Import[:logger, :instrumenter, :exception_notifier, :http]
    class RequestError < StandardError
      attr_reader :response, :request

      def initialize(response)
        @response = response
      end

      def message
        %{Request failed: #{response.status}\n#{response.body&.read}}
      end
    end

    def initialize(url:, **imports)
      super(**imports)
      @url = URI.parse(url)
    end

    def stop
      @task&.wait
      http.close
    end

    # Expects array of Decoder<name, timestamp, tags: {}, values: {}>
    def write(metrics)
      body = encode(metrics)

      @task = Async do
        payload = { client: self.class.name, metrics: metrics, body: body }
        instrumenter.instrument("client.write_metrics", payload) do |payload|
          exception_notifier.extra_context(request: { url: write_url, headers: headers, body: body })
          response = http.post(write_url, headers, body)
          payload[:response] = response
          raise RequestError, response unless (200..299).cover?(response.status)
        rescue StandardError => e
          exception_notifier.capture_exception(e, extra: { response: response, body: response&.read })
          raise
        end
      end
    end

    private

    def write_url
      @write_url ||= begin
        query = URI.encode_www_form(
          db: @url.path.split("/").last,
          precision: "u"
        )

        builder = (@url.scheme == "https" ? URI::HTTPS : URI::HTTP)
        builder.build(host: @url.host, port: @url.port, path: "/write", query: query).to_s
      end
    end

    def headers
      @headers ||= begin
        headers = []
        headers << ["Authorization", "Basic #{Base64.strict_encode64(@url.userinfo)}"] if @url.userinfo
        headers
      end
    end

    def encode(metrics)
      Logsnarf::Encoders::Influxdb.new(metrics).call
    end
  end
end
