package credentials

import (
	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/aws/aws-sdk-go/service/dynamodb"
	"github.com/aws/aws-sdk-go/service/dynamodb/dynamodbattribute"
	"github.com/rs/zerolog/log"
)

// DynamoDBAdapter fuckoff go
type DynamoDBAdapter struct {
	client *dynamodb.DynamoDB
}

type item struct {
	Name        string            `json:"name"`
	Type        string            `json:"type"`
	Credentials map[string]string `json:"credentials"`
}

// New fuckoff go
func New() *DynamoDBAdapter {
	sess := session.Must(session.NewSessionWithOptions(session.Options{
		SharedConfigState: session.SharedConfigEnable,
	}))
	config := aws.NewConfig().WithLogLevel(aws.LogDebugWithHTTPBody)
	return &DynamoDBAdapter{
		client: dynamodb.New(sess, config),
	}
}

// Request fuckoff go
func (a *DynamoDBAdapter) Request(token string) (creds Credentials, err error) {
	result, err := a.client.GetItem(&dynamodb.GetItemInput{
		TableName: aws.String("logsnarf_config"),
		Key: map[string]*dynamodb.AttributeValue{
			"token": {
				S: aws.String(token),
			},
		},
	})

	if err != nil {
		log.Error().Err(err).Msg("")
		return creds, err
	}

	var i item
	err = dynamodbattribute.UnmarshalMap(result.Item, &i)
	if err != nil {
		log.Error().Err(err).Msg("Failed to unmarshal credentials")
	}

	if i.Name == "" {
		log.Error().Err(err).Msgf("Could not find credentials for token %s", token)
		return
	}

	log.Debug().Interface("Item", i).Msg("")

	creds = Credentials{
		Token: token,
		Name:  i.Name,
		Type:  i.Credentials["type"],
		Secrets: Secrets{
			URL: i.Credentials["influxdb_url"],
		},
	}

	return creds, nil
}
