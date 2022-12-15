# frozen_string_literal: true

App.register_provider :dynamodb do
  prepare do
    require "aws-sdk-dynamodb"
  end

  start do
    register(:dynamodb, Aws::DynamoDB::Client.new(logger: App[:logger]))
  end
end
