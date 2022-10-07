# frozen_string_literal: true

App.register_provider :instrumentation do
  prepare do
    require "dry/monitor/notifications"
  end

  start do
    require App.root.join "lib/freedom_patches/notifications"

    App[:notifications].register_event("extractor.extract")
    App[:notifications].register_event("client.write_metrics")

    require App.root.join "lib/subscribers/extractor"
  end
end
