# frozen_string_literal: true

require "import"

module Logsnarf
  module Adapter
    class InfluxdbV1
      include Import[:logger, :instrumenter]

      class RequestError < StandardError
        attr_reader :response, :request

        def initialize(response)
          @response = response
        end

        def message
          %{Request failed: #{response.status}\n#{response.body&.read}}
        end
      end

      def initialize(creds, **imports)
        super(**imports)
        @creds = creds
        @logger = logger.with(name: "influxdb_v1 #{creds['name']}")
        @url = URI.parse(@creds.dig("credentials", "influxdb_url"))
        @client = Logsnarf::Clients::InfluxdbV1.new(url: @url, logger: @logger)
      end

      def stop
        logger.debug "Adapter stopping"
        @client.stop
      end

      def write_metrics(metrics)
        @client.write(metrics)
      end
    end
  end
end
