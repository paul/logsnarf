# frozen_string_literal: true

class Application
  include Import[:credentials_store, :adapter_store, :extractor, :notifier]
  include Dry::Monads[:result]
  include Dry::Monads::Do.for(:ingress)

  def ingress(token, io)
    creds = yield credentials_store.get(token).or(Failure[:credentials])
    Sentry.set_user(id: token, name: creds.name)
    adapter = yield adapter_store.get(creds)
    metrics = yield extractor.extract(io)
    adapter.push(metrics)
  end
end
