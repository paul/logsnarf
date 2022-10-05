# frozen_string_literal: true

class Cache
  include Import[:logger]

  def initialize(limit: 1024, maximum_size: 1024 * 64, flush_interval: 60, **deps)
    super(**deps)

    @limit = limit
    @maximum_size = maximum_size
    @flush_interval = flush_interval

    @index = {}
    @hit = 0
    @miss = 0
    @pruned = 0

    # @gardener = Async(transient: true, annotation: self.class) do |task|
    #   loop do
    #     task.sleep(@flush_interval)

    #     pruned = flush
    #     @pruned += pruned
    #     logger.info
    #   end
    # end
  end

  def close
    @gardener.stop
  end

  attr :index

  def getset(key, &request)
    if value = @index[key]
      @hit += 1
    else
      @miss += 1
      value = request.call
      index[key] = value
    end
    value
  end
end
