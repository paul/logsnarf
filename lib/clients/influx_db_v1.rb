# frozen_string_literal: true

module Clients
  class InfluxDbV1
    include Import[:logger, :notifier, :notifications, :http, encoder: "encoders.influx_db"]
    class RequestError < StandardError
      attr_reader :response, :body

      def initialize(response, body = nil)
        @response, @body = response, body
        super(message)
      end

      def message
        %{Request failed: #{response.status} => #{body}}
      end
    end

    def initialize(url:, **deps)
      super(**deps)
      @url = URI.parse(url)
    end

    def stop
      @task&.wait
      http.close
    end

    # Expects array of Decoder<name, timestamp, tags: {}, values: {}>
    def write(metrics)
      txn = Sentry.start_transaction(op: "write_metrics")

      body = encoder.encode(metrics)

      @task = Async do
        payload = { client: self.class.name, metrics:, body: }
        txn.start_child(op: "#{self.class.name}.request") do |span|
          notifications.instrument("client.write_metrics", payload) do |payload|
            notifier.set_context(:request, { url: write_url, headers:, body: })
            span.set_data(:url, write_url)
            span.set_data(:request_body, body)
            response = http.post(write_url, headers, body)
            payload[:response] = response
            span.set_data(:response_status, response.status)
            response_body = response.finish&.read
            span.set_data(:response_body, response_body)
            raise RequestError.new(response, response_body) unless response.success?
          rescue StandardError => e
            notifier.capture_exception(e)
            logger.failure(self, e)
          ensure
            response&.close
          end
        end
      end
    ensure
      txn.finish
    end

    private

    def write_url
      @write_url ||= begin
        query = URI.encode_www_form(
          db: @url.path.split("/").last,
          precision: "u"
        )

        builder = (@url.scheme == "https" ? URI::HTTPS : URI::HTTP)
        builder.build(host: @url.host, port: @url.port, path: "/write", query:).to_s
      end
    end

    def headers
      @headers ||= begin
        headers = []
        headers << ["Authorization", "Basic #{Base64.strict_encode64(@url.userinfo)}"] if @url.userinfo
        headers.freeze
      end
    end
  end
end
