#!/bin/bash
set -e

parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )

secret_key_path=$parent_path/../../.secrets/stripe/secret_key
if [ ! -f $secret_key_path ]; then
    echo "secret_key file does not exist. Follow the setup instructions first. Path: $secret_key_path"
    exit 1
fi

stripe listen \
      --forward-to localhost:3000/api/billing/stripe/webhook \
      --api-key=$(cat $secret_key_path) \
      --events=checkout.session.completed,invoice.paid,customer.subscription.created,customer.subscription.deleted,customer.subscription.paused,customer.subscription.resumed,customer.subscription.updated
