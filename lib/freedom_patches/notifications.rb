# frozen_string_literal: true

# patch Dry::Notifications to yield the payload to the block, so additional
# fields can be added to it
module Dry
  module Monitor
    class Notifications
      def instrument(event_id, payload = EMPTY_HASH)
        result, time = @clock.measure { yield payload } if block_given?

        process(event_id, payload) do |event, listener|
          if time
            listener.call(event.payload(payload.merge(time:)))
          else
            listener.call(event)
          end
        end

        result
      end
    end
  end
end
