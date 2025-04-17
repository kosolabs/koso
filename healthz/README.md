# HealthZ

This is a cronjob that checks the status of Koso.

## How to Deploy a New Version

1. Build and tag the Docker image.

   ```shell
   docker build --platform linux/amd64 -t us-west1-docker.pkg.dev/koso-429022/healthz/healthz -f healthz/Dockerfile .
   ```

1. Authenticate. Must have the `Service Account Token Creator` role set.

   ```shell
   gcloud auth print-access-token \
   --impersonate-service-account  healthz-docker-push@koso-429022.iam.gserviceaccount.com | docker login \
   -u oauth2accesstoken \
   --password-stdin https://us-west1-docker.pkg.dev

   # Alternatively, use a service account key: cat <service-account-key.json> | docker login -u _json_key --password-stdin https://us-west1-docker.pkg.dev
   ```

1. Push to Artifact Registry.

   ```shell
   docker push us-west1-docker.pkg.dev/koso-429022/healthz/healthz
   ```

1. Update the [Container image URL on Google Cloud Console](https://console.cloud.google.com/run/deploy/us-west1/koso-healthz?project=koso-429022).

1. Click the Deploy button.
