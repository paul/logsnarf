# frozen_string_literal: true

require "amazing_print"

class CredentialsStore
  include Dry::Monads[:maybe]
  include Import[:logger, :dynamodb, :cache]

  def initialize(**deps)
    super(**deps)
    @cache = {}
    @locks = Hash.new { |h, k| h[k] = Thread::Mutex.new }
  end

  def get(token)
    item = nil
    loop do
      item = try_get(token)
      logger.info "Got creds" unless item.nil?
      break item unless item.nil?

      logger.info "Another thread is fetching creds, will try again later"
      sleep 0.1
    end
    item.creds
  end

  def try_get(token)
    item = @cache[token]
    if item.nil? || item.expired?
      if @locks[token].try_lock

        item = fetch(token)
        @cache[token] = item

        @locks[token].unlock
        item
      end
    else
      item
    end
    # ensure
    #   @locks[token].unlock
  end

  def fetch(token, now = Time.now)
    logger.info "Fetching creds for token #{token}"
    data = @dynamodb
           .get_item(table_name: "logsnarf_config", key: { token: })
           .item

    logger.info "Done fetching creds for token #{token}"
    creds = Maybe(data).fmap { |data| Credentials.new(data) }

    Item.new(creds, now)
  end

  class Item
    include Import[:settings]

    attr :creds, :fetched_at

    def initialize(creds, fetched_at, **deps)
      super(**deps)

      @creds, @fetched_at = creds, fetched_at
      @lock = Thread::Mutex.new
    end

    def expired?(now = Time.now)
      @fetched_at + settings.credentials_cache_ttl < now
    end
  end
end
