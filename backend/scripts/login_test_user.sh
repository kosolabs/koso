
encode() {
    echo "$1" | base64 | tr '/+' '_-' | tr -d '='
}

EMAIL="${1:-user-dev@test.koso.app}"
NAME="${2:-Dev User}"
PICTURE="${3:-https://upload.wikimedia.org/wikipedia/commons/d/d3/150-man-factory-worker-2.svg}"
EXPIRATION="${4:-$(( $(date +%s) + 24*60*60 ))}"

HEADER="{\"alg\":\"HS256\", \"typ\":\"JWT\", \"kid\":\"koso-integration-test\"}"
PAYLOAD="{\"email\":\"$EMAIL\",\"name\": \"$NAME\", \"picture\": \"$PICTURE\", \"exp\": $EXPIRATION}"
SIGNATURE="test_signature_cannot_validate"
TOKEN="$(encode "$HEADER").$(encode "$PAYLOAD").$(encode "$SIGNATURE")"

curl -s -X POST -H "Authorization: Bearer $TOKEN" localhost:3000/api/auth/login
curl -s -X POST -H "Authorization: Bearer $TOKEN" localhost:3000/api/dev/invite_test_user
curl -s -X POST -H "Authorization: Bearer $TOKEN" localhost:3000/api/dev/cleanup_test_data

echo "Logged in as $EMAIL. Token:\n$TOKEN"
