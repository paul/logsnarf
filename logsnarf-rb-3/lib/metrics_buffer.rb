# frozen_string_literal: true

class MetricsBuffer
  include Import[:logger, :settings]
  def initialize(adapter, **deps)
    super(**deps)

    @adapter = adapter

    @buffer = []
    @lock = Thread::Mutex.new

    @gardener = Async(transient: true, annotation: self.class) do |task|
      loop do
        task.sleep(settings.metric_buffer_flush_interval)

        flush
      end
    end
  end

  def push(metrics)
    @lock.synchronize do
      @buffer.concat(metrics)
    end
    flush if full?
  end

  def flush
    return if empty?

    metrics = nil
    @lock.synchronize do
      metrics = @buffer.dup
      @buffer.clear
    end
    @adapter.write(metrics)
  end

  def full?
    @buffer.size > settings.metric_buffer_max_size
  end

  def empty?
    @buffer.empty?
  end
end
