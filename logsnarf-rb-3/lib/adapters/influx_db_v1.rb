# frozen_string_literal: true

module Adapters
  class InfluxDbV1
    include Import[:logger]
    include Dry::Monads[:result]

    attr :client

    def initialize(creds:, **deps)
      super(**deps)
      @creds = creds
      @url = creds.credentials.url
      @buffer = MetricsBuffer.new(self)
      @client = App["clients.influx_db_v1"].new(url: @url)
    end

    def push(metrics)
      @buffer.push(Array(metrics))
      Success()
    end

    def write(metrics)
      client.write(metrics)
    end
  end
end
