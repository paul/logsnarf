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
      sentry.sample_rate = 1.0
    end

    register(:rack_notifier, Sentry::Rack::CaptureExceptions)
    register(:notifier, Sentry)
  end
end
