package credentials

import (
	"github.com/hashicorp/golang-lru"
)

// Secrets is the auth secrets needed to connect
type Secrets struct {
	URL string
}

// Credentials holds the name and secrets for the tsdb adapter
type Credentials struct {
	Token   string
	Name    string
	Type    string
	Secrets Secrets
}

var (
	// Cache is the cache implementation to use
	Cache *lru.Cache

	// Adapter to lookup credentials
	Adapter credentialsAdapter
)

func init() {
	c, err := lru.New(1)
	if err != nil {
		panic("Failed to initialize credentials cache!")
	}

	Cache = c

	Adapter = New()
}

// Fetch will retreive Creds from the cache or do a lookup
func Fetch(token string) (Credentials, error) {
	creds, ok := Cache.Get(token)
	if ok {
		return creds.(Credentials), nil
	}

	creds, err := get(token)
	if err != nil {
		return Credentials{}, err
	}

	return creds.(Credentials), nil
}

func get(token string) (creds Credentials, err error) {
	creds, err = Adapter.Request(token)

	if err != nil {
		return creds, err
	}

	return creds, nil
}
