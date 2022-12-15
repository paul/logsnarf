# frozen_string_literal: true

App.register_provider :sentry do
  prepare do
    require "sentry-ruby"
  end

  start do
    target.start :settings

    Sentry.init do |sentry|
      sentry.dsn = App[:settings].sentry_dsn

      sentry.enabled_environments = App[:settings].sentry_environments
      sentry.environment = App.env
      sentry.logger = App[:logger]

      # Not thread-safe at all
      sentry.traces_sample_rate = false
    end

    register(:rack_notifier, Sentry::Rack::CaptureExceptions)
    register(:notifier, Sentry)
  end
end
