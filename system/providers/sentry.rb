# frozen_string_literal: true

App.register_provider :sentry do
  prepare do
    require "sentry-ruby"
  end

  start do
    Sentry.init do |sentry|
      sentry.dsn = ENV["SENTRY_DSN"]
      sentry.logger = App[:logger]
      sentry.sample_rate = 1.0
    end

    register(:rack_notifier, Sentry::Rack::CaptureExceptions)
  end
end
