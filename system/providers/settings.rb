# frozen_string_literal: true

require "dry/system/provider_sources"

App.register_provider(:settings, from: :dry_system) do
  before :prepare do
    require "./lib/types"
  end

  settings do
    setting :sentry_dsn
    setting :sentry_environments, default: %w[development production], constructor: Types::Array.of(Types::String)
    setting :sentry_sample_rate, default: "0.1", constructor: Types::Coercible::Float

    setting :logger_level, default: App.env == "production" ? :info : :debug, constructor: Types::Coercible::Symbol
      .constructor { |value| value.to_s.downcase.to_sym }
      .enum(:trace, :unknown, :error, :fatal, :warn, :info, :debug)

    setting :credentials_cache_ttl, default: 600, constructor: Types::Coercible::Integer

    setting :metric_buffer_flush_interval, default: 30, constructor: Types::Coercible::Integer
    setting :metric_buffer_max_size, default: 100, constructor: Types::Coercible::Integer
  end
end
