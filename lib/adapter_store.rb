# frozen_string_literal: true

class AdapterStore
  include Dry::Monads[:result]

  def initialize(**deps)
    super(**deps)
    @adapters = {}
  end

  def get(creds)
    @adapters[creds.token] ||= App["adapters.#{Dry::Core::Inflector.underscore(creds.type)}"].new(creds:).then { |r| Success(r) }
  end
end
