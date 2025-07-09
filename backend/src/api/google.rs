use crate::{
    api::{ApiResult, IntoApiResult as _, unauthenticated_error},
    settings::settings,
};
use anyhow::{Result, anyhow};
use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

pub const TEST_USER_SUFFIX: &str = "@test.koso.app";

#[derive(Clone)]
pub struct KeySet {
    inner: Arc<KeySetInner>,
}

struct KeySetInner {
    certs: Mutex<Certs>,
    client: reqwest::Client,
    last_load: Mutex<Instant>,
    enable_test_creds: bool,
}

impl KeySet {
    const LOAD_DEBOUNCE_DURATION: Duration = Duration::from_secs(306);
    const INTEG_TEST_KID: &'static str = "koso-integration-test";

    pub(crate) async fn new() -> Result<KeySet> {
        let distant_past =
            Instant::now() - KeySet::LOAD_DEBOUNCE_DURATION - Duration::from_secs(60 * 60);
        let key_set = KeySet {
            inner: Arc::new(KeySetInner {
                certs: Mutex::new(Certs {
                    keys: Vec::with_capacity(0),
                }),
                client: reqwest::Client::new(),
                last_load: Mutex::new(distant_past),
                enable_test_creds: settings().is_dev(),
            }),
        };
        // Avoid a cold start by preloading keys initially.
        let _ = key_set.load_keys(None).await?;
        Ok(key_set)
    }

    async fn get(&self, kid: &str) -> Result<DecodingKey> {
        if kid == KeySet::INTEG_TEST_KID {
            if !self.inner.enable_test_creds {
                return Err(anyhow!(
                    "Tried to fetch key for test creds ({kid}) but test creds aren't enabled."
                ));
            }
            return Ok(DecodingKey::from_rsa_components("MA", "MA")?);
        }

        // This is the happy path. The key is already cached and ready to go.
        {
            let certs = self.inner.certs.lock().await;
            if let Some(key) = certs.get_key(kid) {
                return Ok(key);
            }
        }

        // Maybe the key is new, so try reloading from Google.
        // NOTE: we might also reload keys periodically to better handle key revocation
        // but this works well enough in practice.
        match self.load_keys(Some(kid)).await? {
            Some(key) => Ok(key),
            None => Err(anyhow!("Key not found: {kid}")),
        }
    }

    async fn load_keys(&self, kid: Option<&str>) -> Result<Option<DecodingKey>> {
        let mut last_load = self.inner.last_load.lock().await;
        // Limit how often we make the remote call to avoid being DOS'd.
        // If we didn't find the cert a minute ago, it probably still doesn't exist.
        if Instant::now() - *last_load < KeySet::LOAD_DEBOUNCE_DURATION {
            return if let Some(kid) = kid {
                Ok(self.inner.certs.lock().await.get_key(kid))
            } else {
                Ok(None)
            };
        }

        let json = self
            .inner
            .client
            .get("https://www.googleapis.com/oauth2/v3/certs")
            .send()
            .await?
            .text()
            .await?;
        let certs = Certs::parse(&json)?;
        tracing::debug!("Fetched google certs: {certs:?}");

        let key = kid.and_then(|kid| certs.get_key(kid));
        *last_load = Instant::now();
        *self.inner.certs.lock().await = certs;
        drop(last_load);

        Ok(key)
    }
}

struct Key {
    kid: String,
    key: DecodingKey,
}

#[derive(Debug)]
struct Certs {
    keys: Vec<Key>,
}

impl Certs {
    fn get(&self, kid: &str) -> Option<&Key> {
        self.keys.iter().find(|&key| key.kid == *kid)
    }

    fn get_key(&self, kid: &str) -> Option<DecodingKey> {
        self.get(kid).map(|k| k.key.clone())
    }

    fn parse(json: &str) -> Result<Certs> {
        let raw_certs: RawCerts = serde_json::from_str(json)?;

        let mut keys = Vec::with_capacity(raw_certs.keys.len());
        for key in raw_certs.keys {
            keys.push(Key {
                kid: key.kid,
                key: DecodingKey::from_rsa_components(&key.n, &key.e)?,
            })
        }
        Ok(Certs { keys })
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.kid)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RawKey {
    kid: String,
    alg: String,
    n: String,
    e: String,
    kty: String,
    r#use: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawCerts {
    keys: Vec<RawKey>,
}

#[tracing::instrument(skip(request, next), fields(email))]
pub(crate) async fn authenticate(mut request: Request, next: Next) -> ApiResult<Response<Body>> {
    let key_set = request.extensions().get::<KeySet>().unwrap();
    let headers = request.headers();

    let bearer = if let Some(auth_header) = headers.get("Authorization") {
        let Ok(auth) = auth_header.to_str() else {
            return Err(unauthenticated_error(&format!(
                "Could not convert auth header to string: {auth_header:?}"
            )));
        };
        let parts: Vec<&str> = auth.split(' ').collect();
        if parts.len() != 2 || parts[0] != "Bearer" {
            return Err(unauthenticated_error(&format!(
                "Could not split bearer parts: {parts:?}"
            )));
        }
        parts[1]
    } else if let Some(swp_header) = headers.get("sec-websocket-protocol") {
        let Ok(swp) = swp_header.to_str() else {
            return Err(unauthenticated_error(&format!(
                "sec-websocket-protocol must be only visible ASCII chars: {swp_header:?}"
            )));
        };
        // Search the comma separated parts for "bearer"
        // and return the subsequent part containing the token value.
        let mut iter = swp.split(", ");
        let token = loop {
            match iter.next() {
                None => break None,
                Some("bearer") => break iter.next(),
                Some(_) => {
                    continue;
                }
            }
        };
        let Some(token) = token else {
            return Err(unauthenticated_error(&format!(
                "sec-websocket-protocol must contain a bearer token: {swp:?}"
            )));
        };

        token
    } else {
        return Err(unauthenticated_error("Authorization header is absent."));
    };

    let Ok(header) = jsonwebtoken::decode_header(bearer) else {
        return Err(unauthenticated_error(&format!(
            "Could not decode header: {bearer:?}"
        )));
    };
    let Some(kid) = header.kid else {
        return Err(unauthenticated_error(&format!(
            "header.kid is absent: {header:?}"
        )));
    };
    let key = key_set
        .get(&kid)
        .await
        .context_unauthenticated("certs is absent")?;

    let mut user = if kid == KeySet::INTEG_TEST_KID {
        decode_and_validate_test_token(bearer, &key)?
    } else {
        decode_and_validate_token(bearer, &key)?
    };
    // Canonicalize emails to lower case.
    // Why? In Oct. 2024 Google suddenly started serving emails
    // with upper case characters, where previously they were lower.
    user.email = user.email.to_lowercase();

    tracing::Span::current().record("email", user.email.clone());
    assert!(request.extensions_mut().insert(user).is_none());

    Ok(next.run(request).await)
}

fn decode_and_validate_token(token: &str, key: &DecodingKey) -> ApiResult<User> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    // Allow the token to last seven days longer than the given expiry.
    // This number matches the clients's validation in auth.ts.
    const SEVEN_DAYS_SECS: u64 = 7 * 24 * 60 * 60;
    validation.leeway = SEVEN_DAYS_SECS;
    validation.set_audience(&[
        "560654064095-kicdvg13cb48mf6fh765autv6s3nhp23.apps.googleusercontent.com",
    ]);
    validation.set_issuer(&["https://accounts.google.com"]);
    let token = jsonwebtoken::decode::<User>(token, key, &validation)
        .context_unauthenticated("Failed validation")?;
    if token.claims.email.is_empty() {
        return Err(unauthenticated_error(&format!(
            "Claims email is empty: {token:?}"
        )));
    }
    Ok(token.claims)
}

fn decode_and_validate_test_token(token: &str, key: &DecodingKey) -> ApiResult<User> {
    // Example Jwt:
    //   eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Imtvc28taW50ZWdyYXRpb24tdGVzdCJ9.eyJlbWFpbCI6InRlc3RAdGVzdC5rb3NvLmFwcCIsIm5hbWUiOiJQb2ludHktSGFpcmVkIEJvc3MiLCJwaWN0dXJlIjoiaHR0cHM6Ly9zdGF0aWMud2lraWEubm9jb29raWUubmV0L2RpbGJlcnQvaW1hZ2VzLzYvNjAvQm9zcy5QTkciLCJleHAiOjIwMjQ3ODgwMTR9.3btheBY5h0nQRpWNODfYWQ_mMc26551178jrSDmpv_c
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.insecure_disable_signature_validation();
    let token = jsonwebtoken::decode::<User>(token, key, &validation)
        .context_unauthenticated("Failed to decode test cred token")?;

    let user = token.claims;
    if !user.email.ends_with(TEST_USER_SUFFIX) {
        return Err(unauthenticated_error(&format!(
            "Invalid test cred email: {}",
            user.email
        )));
    }
    Ok(user)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct User {
    pub(crate) email: String,
    pub(crate) name: String,
    pub(crate) picture: String,
    pub(crate) exp: usize,
}

#[cfg(test)]
mod tests {
    use crate::api::google::{
        Certs, KeySet,
        test_utils::{KID_1, KID_2, testonly_key_set},
    };
    use std::time::{Duration, Instant};
    use tokio::{task::yield_now, time::sleep};

    const TEST_KID: &str = "load_does_not_block_get_of_existing_keys_kid";

    fn certs() -> Certs {
        Certs::parse(include_str!("../testdata/certs.json")).unwrap()
    }

    #[tokio::test]
    async fn fetch_succeeds() {
        let key_set: Result<KeySet, anyhow::Error> = KeySet::new().await;
        assert!(key_set.is_ok());
        let key_set = key_set.unwrap();

        assert!(key_set.get("does_not_exist_kid").await.is_err());

        let kid = key_set
            .inner
            .certs
            .lock()
            .await
            .keys
            .first()
            .unwrap()
            .kid
            .clone();
        assert!(key_set.get(&kid).await.is_ok());
    }

    #[tokio::test]
    async fn load_does_not_block_get_of_existing_keys() {
        let key_set = testonly_key_set().await.unwrap();
        // Simulate a running call to load remote keys.
        // This will block any calls to fetch keys that don't exist, but will NOT
        // block fetches of keys that are already loaded.
        let mut last_load = key_set.inner.last_load.try_lock().unwrap();
        *last_load = Instant::now();

        assert!(key_set.get(KID_1).await.is_ok());
        assert!(key_set.get(KID_2).await.is_ok());

        // Spawn a task to fetch a key that does not and will not exist.
        // This will block on `last_load` being dropped.
        let not_found = {
            let key_set = key_set.clone();
            tokio::spawn(async move { key_set.get("does_not_exist_kid").await })
        };
        // Spawn a task to fetch a key that does not exist but will exist after load.
        // This will block on `last_load` being dropped.
        let found = {
            let key_set = key_set.clone();
            tokio::spawn(async move { key_set.get(TEST_KID).await })
        };
        yield_now().await;
        sleep(Duration::from_millis(50)).await;
        assert!(!not_found.is_finished());
        assert!(!found.is_finished());

        // Insert a new key, simulating loading of a new key.
        {
            let mut certs = key_set.inner.certs.lock().await;
            let key = certs.keys.first().unwrap().key.clone();
            certs.keys.push(crate::api::google::Key {
                kid: TEST_KID.to_string(),
                key,
            });
        }
        drop(last_load);

        let Err(error) = not_found.await.unwrap() else {
            panic!("not_found call unexpectedly succeeded");
        };
        assert!(error.to_string().contains("Key not found"));

        assert!(found.await.unwrap().is_ok());
    }

    #[test]
    fn get_returns_error_if_kid_is_missing() {
        let certs = certs();
        assert!(certs.get("missing").is_none())
    }

    #[test]
    fn get_returns_key_if_kid_exists() {
        let certs = certs();
        assert!(certs.get("1").is_some());
        let key = certs.get("1").unwrap();
        assert!(key.kid == "1");
    }
}

#[cfg(test)]
pub(crate) mod test_utils {
    use super::{Certs, Key, KeySet, KeySetInner};
    use anyhow::Result;
    use jsonwebtoken::{DecodingKey, EncodingKey, Header};
    use rsa::{
        RsaPrivateKey,
        pkcs1::{DecodeRsaPrivateKey as _, EncodeRsaPublicKey as _},
    };
    use serde::{Deserialize, Serialize};
    use std::{
        collections::HashMap,
        sync::Arc,
        time::{Duration, Instant},
    };
    use tokio::sync::Mutex;

    pub(crate) const KID_1: &str = "kid_1";
    pub(crate) const PEM_1: &str = "-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEApoqzB090M6ZLQAufcb4JMFCu5WQ0HITPHQcvcrtmv2nBlcXF
Ml4g08B47b0yL73KhJA20GRY19UVQsuUuSNvaYsIvHfGE0KuN+OWIfrA2TnwLBNH
0VqbIiar9hAluGt0OOcd4itx0dqqPq4VecO7qwgzjtf6u+3AZCCHxbLqNIEGUPQ1
HBUuWMnRGgubjGdLxR3fujpmoz3LaeDngOc6NwesAbVU2kXOQDwg6pJOFjpUJ537
zQaAvHiOz6JmvXvqUZycVngPqpykYG0Qc4qKZN0uRRLS7hX6HRT8pyksEQHscwsf
+UfkOlcpf7QLwc1+3B++7/GU+AiRPizioaneywIDAQABAoIBAAob3SYScpE4BNVM
X08+I7ylCRivbmJUxWWTrBOgwGnZ94Ap0wBtqjxwMGbt1wAC2yoUvt8DWEkxi+rU
BKEAP6D+aXdXJdTBdWW7EL/bQp1s4OEsAm34u/Xktwdmj4OUMifKD4xM6sm8Jh1L
383WTaviAY8oGPYTRlxNhTBA3ep12VozlFjz67tQJUdqNvNyww9DlqXKQXTlBQrG
zCbIDNczqUACKEVQfg7fGsbq8TWVXH+GPMVORLzd9mG/EHLe6Bt3/D1p88h6s5NO
rFRBPbrD68BPNGW17RH/G+giQDLTNMj3Mz230foLTPwbsXTkdPxFsinK0k2dGSXM
9aDUMTECgYEA2vM4JCunine5LaBe9KmKtoZ8OOUposZuBNABt1lLuQb9/vcoqRNv
+Vlidor9mRXslJh70QnI2P+iERnrj3ygQQSU0GC9vDdnV68H3U0+fqygBYf0U9+q
lskJfubvZmXVfbwA2KDq6B/JzUe2qFnQnJSijG3Up4qMs9nfWdmIwsUCgYEAwrkv
nEuqN69kQrcZxcw6P+jeyyr79n/al2DdU5OUlCflm9DMPBedXogL4lwftBD/vvy+
yRpPKB1z0ghAasG8c3oSf4vJ84jtvAZTlxiNjyFJe1q5izmEDLZ+sOWKc1rl7Pdl
IhbpS8hatjk7tf+9cFNqP8xQZ4sgDA4PypCq9E8CgYEAqBUFKUdWBAeq4er2Wm55
LWwKmwbZsrsQJKOmXaGwbud+P6hvz3Q7hrlmzEghLM9W4jA5BR200VlVijlSy8FJ
qQAiWeGaZo5FyFt29x3gdxCAfB6Fo4nWBJFqt8ADUqGkhjS4lZTbIL2ehvehspXY
fwvfyVxbXw8OutbsDqbfxV0CgYBMRguvNjhDvbERLPWsc/XxKL90Z67wfF5cY3Xu
keVmL0aSRTRq6XkcGUBGd313pBz5a7kzvtl4xiijAdZxutediBiM223MtjshJn1B
tz1j7k8BQaViMrJV5Ho1woP78YQU0UdNFhpmM+HMdRi9jqJeyF3bBaYNGQMBldR7
rTU5owKBgFn8VLurbLwwl474PyRuw367/J3Rrsccz5OR0MB2pazhQO+rVZqi4wcj
ptl5DBYOjW/AmvCqnAX8BUX1OgBSk3GrR4EBAw3itn0rr8kGek5salD7cyqVQ7pY
JTE6ixYn3f+aGTuhJ7dIxl1hUvhfulNB2R8w1qQuuzSkegHjQbiM
-----END RSA PRIVATE KEY-----";

    pub(crate) const KID_2: &str = "kid_2";
    pub(crate) const PEM_2: &str = "-----BEGIN RSA PRIVATE KEY-----
MIIEpgIBAAKCAQEAwg7lRJdetSD4rfbu1r5fMl8C0Pp3jtoO6bmrzO/cM55nEzU7
a5nf5M/kCT9/tZM0cUP92hR9o+vKtd6VO9O+MCG5fPnkbXmi0KJivQDJ3CD42Ukv
8NYSNJPcN9KBdGjquvJ6OdxveQgcSBMgVpHf8J1E4KapY/+SYYVprfX4JpY9+Aux
whDM589Iadzl5R+CdOoRNgpW1dvsMXaJ5FQK5ftIcg4M2qpm860KxZe3S7l4/A/H
0Hni8nPSG7uRtr1V6b1HO8r8XF1jGULNLjv8ktF4RYWY8B7EPnvudfEraDZ5YD9m
8ySlcX9ig7Rg13gbm7p0ZJMvttr29zTtdMCSKwIDAQABAoIBAQCSg/6UGBl4dgls
B1lyp09mz5dnwwPLxlWmH/pXg+3kxz8ZoIJZjlceAdwxI1E//YGF1wjtw7TMs7Vc
NU7FWexpmLzcYCwYf9Lu5PvZqaO+4OIh5AEfO/GI4u5M81GsW56GQZcI4qcDYZ1A
ybgLxJ2opIUhfJO+HXMe0ETnBCQ0tL8c1nbwYwMHEcXNcQ3Qrklok2HhkeE9DlXp
x/6ZshFeFFcwTgKM2YTTf6ucLA7o0FfZeaV5vFLf60WBvZbdVZC/1fSHFzCMoVGZ
jPz7JNgYWLywi8c+QSoW9VY7soCIzCgpbGM77Had/96pcPLCgXUFIt0GchqOQB+L
pGG2vkexAoGBAM0JtcFcm50lIZhxXCKRdXapTltTeSENxqmA3nbiObxG7cH0Kr9Z
ym3R6moreIjTBQD6FaJKrEHLdGEJA4nHQQgzOmrJP1JOGEcBlWj7auXlLh2j8Zm8
cvu6RGs3rW8uNyUqxBINn+i6PyPIEOUKgX3Q1Db8poup/7G7fU7x89tjAoGBAPJK
jyQCjBKocEAVRlHL4+Xgw7ywTlH9Kt21eSt0n9hhd3H4MXZu7U1kBcFuFGzLamde
uo06QgOTWufHS888vgOrhMILuY/jWX+EJruQ7reW/Q41fJeWgeExxDnenVJoQc5a
E+Pv/y+B7vUj/oTfepmZS2UwrHn/HJWWVxjDz/yZAoGBAKcgMglRXfpCKNckF3CJ
1hAJwrfIG2So4PSK+Uo37c2clvHP/wQHwWuwff1aP55vOpXoQrgNW8kpeEwb18l3
I3f2obgnH7kLtNgz30A6JpELNIKufiDMrYCn/FrUgEauif5+lGEOv/gnz41v5u16
mcAe9st3Np2CzMtnQqWVrCp3AoGBAIbL/ljtZdqXhWPRsj6drZvd8WgPunMY43lX
liMcDjYG+7oXeAVI75MH27/iq1Bf10HNTQJ3b/SnTYL3uPCB/cDy8rg2Z7Vqqgcs
kZP2rSjMwtrd3QRFAtszodUESghn4nyYVsqQYiufIT+XF+n6ny3HQE/6xWpWCSQb
8Tbg8dy5AoGBAIklRnELhI9o8R//xaW7xZxEmabD2F2eQpLCh1j8RJABYIkFQ4Y+
Nh2lZEEmIkksR2hKZ2cnnk0tbqR5QsUPbsuUy6Ad08NwA5YwUkYxMkeMmsEG5yWJ
J4Q2mbNL+TM1cM1BbzXn6SBWWTKlyEx7OLgQ2+VlZ1CRLQI/iI1tbHIt
-----END RSA PRIVATE KEY-----";

    pub(crate) async fn testonly_key_set() -> anyhow::Result<KeySet> {
        let mut keys = HashMap::new();

        let priv_key_1 = RsaPrivateKey::from_pkcs1_pem(PEM_1).unwrap();
        let pub_key_1 = priv_key_1.to_public_key();
        let key_1 = DecodingKey::from_rsa_pem(
            pub_key_1
                .to_pkcs1_pem(rsa::pkcs8::LineEnding::CR)
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        keys.insert(KID_1.to_string(), key_1);

        let priv_key_2 = RsaPrivateKey::from_pkcs1_pem(PEM_2).unwrap();
        let pub_key_2 = priv_key_2.to_public_key();
        let key_2 = DecodingKey::from_rsa_pem(
            pub_key_2
                .to_pkcs1_pem(rsa::pkcs8::LineEnding::CR)
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        keys.insert(KID_2.to_string(), key_2);

        new_fake(keys).await
    }

    pub(crate) async fn new_fake(keys: HashMap<String, DecodingKey>) -> Result<KeySet> {
        let distant_future = Instant::now() + Duration::from_secs(60 * 60 * 24);
        let key_set = KeySet {
            inner: Arc::new(KeySetInner {
                certs: Mutex::new(Certs {
                    keys: keys
                        .into_iter()
                        .map(|(kid, key)| Key { kid, key })
                        .collect(),
                }),
                client: reqwest::Client::new(),
                last_load: Mutex::new(distant_future),
                enable_test_creds: false,
            }),
        };
        // Verify load does nothing thanks to the distant last_load time set above.
        let _ = key_set.load_keys(None).await?;
        Ok(key_set)
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) struct Claims {
        pub(crate) aud: String,
        pub(crate) iss: String,
        pub(crate) email: String,
        pub(crate) name: String,
        pub(crate) picture: String,
        pub(crate) exp: u32,
    }

    impl Default for Claims {
        fn default() -> Claims {
            Claims {
                aud: "560654064095-kicdvg13cb48mf6fh765autv6s3nhp23.apps.googleusercontent.com"
                    .to_string(),
                iss: "https://accounts.google.com".to_string(),
                email: "valid-user@koso.app".to_string(),
                name: "Valid User".to_string(),
                picture: "koso.app/valid-user/pic".to_string(),
                exp: 2024788014,
            }
        }
    }

    pub(crate) fn encode_token(claims: &Claims, kid: &str, pem: &str) -> Result<String> {
        let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some(kid.to_string());
        Ok(jsonwebtoken::encode(
            &header,
            claims,
            &EncodingKey::from_rsa_pem(pem.as_bytes())?,
        )?)
    }
}
