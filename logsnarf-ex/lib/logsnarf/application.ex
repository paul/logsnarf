defmodule Logsnarf.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  def start(_type, _args) do
    children = [
      # Starts a worker by calling: Logsnarf.Worker.start_link(arg)
      # {Logsnarf.Worker, arg}
      Plug.Cowboy.child_spec(
        scheme: :http,
        plug: Logsnarf.Endpoint,
        options: [port: 4001]
    ]

    # See https://hexdocs.pm/elixir/Supervisor.html
    # for other strategies and supported options
    opts = [strategy: :one_for_one, name: Logsnarf.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
