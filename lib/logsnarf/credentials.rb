# frozen_string_literal: true

require "lru_redux"
require "aws-sdk-dynamodb"
require "singleton"

module Logsnarf
  class Credentials
    include Singleton

    def initialize
      @cache = LruRedux::TTL::ThreadSafeCache.new(1000, 15 * 60)
      @dynamodb = Aws::DynamoDB::Client.new(log_level: :debug, logger: Logger.new(STDOUT))
      @gets = @misses = 0
    end

    def get(token)
      @gets += 1
      @cache.getset(token) do
        @misses += 1
        @dynamodb.get_item(table_name: "logsnarf_config", key: { token: token }).item&.dig("credentials")
      end
    end

    def stats
      { gets: @gets, misses: @misses }
    end
  end

  @@credentials = Credentials.instance
  def self.credentials
    @@credentials
  end
end
