# frozen_string_literal: true

require "import"

module Logsnarf
  module Adapter
    class InfluxdbV1
      include Import[:logger, :instrumenter]

      def initialize(creds, **imports)
        super(**imports)
        @creds = creds
        @logger = logger.with(name: "influxdb_v1 #{creds['name']}")
        @url = @creds.dig("credentials", "influxdb_url")
        @client = Logsnarf::Clients::InfluxdbV1.new(url: @url, logger: @logger)
      end

      def stop
        logger.debug "Adapter stopping"
        @client.stop
      end

      def write_metrics(metrics)
        @client.write(metrics)
      end

      private

      attr_reader :creds
    end
  end
end
