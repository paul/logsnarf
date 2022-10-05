# frozen_string_literal: true

require "dry/system/provider_sources"

App.register_provider(:settings, from: :dry_system) do
  before :prepare do
    require "./lib/types"
  end

  settings do
    setting :credentials_cache_ttl, default: 5, constructor: Types::Integer

    setting :logger_level, default: :info, constructor: Types::Symbol
      .constructor { |value| value.to_s.downcase.to_sym }
      .enum(:trace, :unknown, :error, :fatal, :warn, :info, :debug)
  end
end
