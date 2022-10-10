# frozen_string_literal: true

class Application
  include Import[:credentials_store, :adapter_store, :extractor, :notifier]
  include Dry::Monads[:result]
  include Dry::Monads::Do.for(:ingress)

  def ingress(token, io)
    txn = Sentry.get_current_scope.get_transaction

    span = txn.start_child(op: :ingress)

    creds = yield credentials_store.get(token).or(Failure[:credentials])
    Sentry.set_user(id: token, name: creds.name)
    adapter = yield adapter_store.get(creds)
    metrics = yield extractor.extract(io)
    adapter.push(metrics)

    span.finish

    Success()
  end
end
