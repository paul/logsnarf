# frozen_string_literal: true

require "lru_redux"
require "aws-sdk-dynamodb"
require "singleton"

module Logsnarf
  class Credentials
    class Store
      include Import[:logger, :instrumenter]
      def initialize(**imports)
        super
        @cache = LruRedux::TTL::ThreadSafeCache.new(1000, 15 * 60)
        @dynamodb = Aws::DynamoDB::Client.new(logger: logger)
        @gets = @misses = 0
      end

      def get(token)
        @gets += 1
        @cache.getset(token) do
          @misses += 1
          config = @dynamodb
                   .get_item(table_name: "logsnarf_config", key: { token: token })
                   .item
          Credentials.new(config) if config
        end
      end
      alias_method :[], :get
      alias_method :fetch, :get

      def stats
        { gets: @gets, misses: @misses }
      end
    end

    extend Forwardable

    delegate [:[], :dig] => :@config

    def initialize(config)
      @config = config
    end

    def type
      @config.dig("credentials", "type")
    end

    def name
      @config["name"]
    end
  end
end
