# HealthZ

This is a cronjob that checks the status of Koso.

## How to Deploy a New Version

1. Build and tag the Docker image.

   ```shell
   docker build --platform linux/amd64 -t us-west1-docker.pkg.dev/koso-429022/healthz/healthz .
   ```

1. Authenticate.

   ```shell
   cat <service-account-key.json> | docker login -u _json_key --password-stdin https://us-west1-docker.pkg.dev
   ```

1. Push to Artifact Registry.

   ```shell
   docker push us-west1-docker.pkg.dev/koso-429022/healthz/healthz
   ```

1. Update the [Container image URL on Google Cloud Console](https://console.cloud.google.com/run/jobs/edit/us-west1/healthz?project=koso-429022).

1. Click the Update button.
