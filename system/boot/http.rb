# frozen_string_literal: true

Logsnarf::App.boot(:http) do
  init do
    require "async/http/internet"
  end

  start do
    register(:http, Async::HTTP::Internet.new)
  end

  stop do
    http.close
  end
end
