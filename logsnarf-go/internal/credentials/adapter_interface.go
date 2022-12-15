package credentials

// Adapter Interface for adapters that can retreive credentials
type credentialsAdapter interface {
	Request(token string) (creds Credentials, err error)
}
