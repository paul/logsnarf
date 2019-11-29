# frozen_string_literal: true

Logsnarf::App.boot(:exception_notifier) do
  init do
    require "types"
    require "raven"
  end

  settings do
    key :environments, Types::Strict::Array.of(Types::Strict::Symbol).default(%i[production].freeze)
    key :environment,  Types::Strict::Symbol
    key :logger,       Types::Any
    key :sentry_dsn,   Types::Strict::String
  end

  configure do |config|
    config.environment = Logsnarf::App.env
    config.logger      = Logsnarf::App[:logger]
    config.sentry_dsn  = ENV["SENTRY_DSN"]
  end

  start do
    Raven.configure do |raven|
      raven.dsn                 = config.sentry_dsn
      raven.environments        = config.environments
      raven.current_environment = config.environment
      raven.logger              = logger
    end

    register(:exception_notifier, Raven)
  end
end
