# frozen_string_literal: true

module Logsnarf::Clients
  class InfluxdbV1
    def initialize(url:, logger: Async.logger, notifier: Raven)
      @url = url
      @logger, @notifier = logger, notifier
      @internet ||= Async::HTTP::Internet.new
      at_exit { @internet.close }
    end

    def stop
      @internet.close
    end

    # Expects array of Decoder<name, timestamp, tags: {}, values: {}>
    def write(metrics)
      body = encode(metrics)

      Async do
        logger.info "Writing %d metrics (%d bytes)" % [metrics.size, body.bytesize]
        notifier.extra_context(request: { url: write_url, headers: headers, body: body })
        response = @internet.post(write_url, headers, body)
        raise RequestError, response unless (200..299).cover?(response.status)
      rescue StandardError => e
        notifier.capture_exception(e, extra: { response: response, body: response&.read })
        raise
      end
    end

    private

    attr_reader :logger, :internet, :notifier

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
