# frozen_string_literal: true

class CredentialsStore
  include Dry::Monads[:maybe]
  include Import[:logger, :dynamodb, :cache, :notifier]

  def initialize(**deps)
    super(**deps)
    @cache = {}
    @locks = Hash.new { |h, k| h[k] = Thread::Mutex.new }
  end

  def get(token)
    item = nil
    loop do
      item = try_get(token)
      logger.debug "Got creds" unless item.nil?
      break item unless item.nil?

      logger.debug "Another thread is fetching creds, will try again later"
      sleep 0.1
    end
    item.creds
  end

  def try_get(token)
    item = @cache[token]
    if item.nil? || item.expired?
      if @locks[token].try_lock
        begin
          item = fetch(token)
          @cache[token] = item

          item
        ensure
          @locks[token].unlock
        end
      end
    else
      item
    end
  end

  def fetch(token, now = Time.now)
    txn = Sentry.get_current_scope.get_transaction
    span = txn.start_child(op: :fetch_credentials)
    logger.info "Fetching creds for token #{token}"
    data = @dynamodb
           .get_item(table_name: "logsnarf_config", key: { token: })
           .item

    logger.info "Done fetching creds for token #{token}"
    creds = Maybe(data).fmap { |data| Credentials.new(data) }

    item = Item.new(creds, now)
    span.finish
    item
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
