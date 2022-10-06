# frozen_string_literal: true

App.register_provider(:http) do
  prepare do
    require "async/http/internet"
  end

  start do
    register(:http, Async::HTTP::Internet.new)
  end

  stop do
    http.close
  end
end
