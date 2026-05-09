---
name: botocore
category: cloud-sdk
description: Low-level core library powering AWS CLI and boto3 SDK, handling service definitions, request signing, response parsing, and API endpoint management for AWS operations.
tags:
- aws
- cloud
- sdk
- api
- botocore
author: AI-generated
source_url: https://github.com/boto/botocore
---

## Concepts

- **Botocore is a Python library, not a standalone CLI tool.** The `aws` CLI command (`awscli` package) is the user-facing command built on top of botocore. When you run `aws s3 ls`, botocore loads service definitions from `$BOTOCORE_DATA_DIR` (or bundled JSON files) to resolve endpoint URLs, sign requests using Signature Version 4, and parse XML/JSON responses into Python dictionaries.
- **Service definitions are JSON files stored per service and API version.** Located in `botocore/data/` inside the package, each service (e.g., `s3`, `ec2`) has a directory of timestamped API version folders containing `service-2.json` and `paginators-1.json`. These files define available operations, parameter types, response structure, and error codes.
- **Request signing uses AWS Signature Version 4.** Every API call must include an Authorization header built from: Access Key ID, Secret Access Key, region, service name, timestamp (ISO 8601), and a canonical request hash. If botocore cannot find AWS credentials, all operations raise `botocore.exceptions.NoCredentialsError`.
- **Retry logic and throttling** are governed by `botocore/retryhandler.py` and exponential backoff. When a service returns HTTP 429 or 5xx, botocore retries up to `max_attempts` (default 5) with jitter unless `--no-sign-request` or `AWS_CONFIG_FILE` disables it.
- **Endpoint resolution uses a resolver chain.** First, it checks `endpoint_url` in client kwargs, then `AWS_ENDPOINT_URL` environment variable, then `endpoint_url` in shared config file (`~/.aws/config`), and finally falls back to the global endpoint pattern defined in service JSON.

## Pitfalls

- **Using botocore directly instead of boto3 for high-level operations.** Directly using `botocore.Session().create_client('s3')` returns raw response dictionaries with untyped keys, whereas boto3 provides resource objects and paginators. This leads to fragile code that breaks on API updates.
- **Assuming credentials are found when they are missing.** `botocore` silently reads from `~/.aws/credentials`, `AWS_ACCESS_KEY_ID`/`AWS_SECRET_ACCESS_KEY` env vars, or EC2 instance profile. If none exist, errors appear only at first request, not at client creation, causing delayed failures in long-running scripts.
- **Hardcoding region names that become unavailable.** AWS can deprecate regions (e.g., `us-east-1` vs. `us-east-2`). Botocore's endpoint resolver will raise `DataNotFoundError` or `UnknownEndpointError` if the region is not in the service definition JSON, especially after botocore upgrades.
- **Not specifying a Content-MD5 header for S3 multipart uploads.** When uploading parts to S3 via botocore's `UploadPart` operation, the service requires an `Content-MD5` header for each part. Omitting this causes HTTP 400 Bad Request with error code `InvalidDigest`, and the entire multipart upload becomes invalid.
- **Passing incorrect datetime formats to timestamp parameters.** Botocore expects ISO 8601 format (`YYYY-MM-DDTHH:MM:SSZ`) for parameters like `DeletionTime` in S3 lifecycle rules. Passing Python `datetime` objects directly without `.isoformat()` causes serialization errors or silent timezone misinterpretations.

## Examples

### List available AWS services and their latest API versions registered with botocore
**Args:** `python -c "import botocore; import os; print(os.listdir(botocore.__path__[0] + '/data/'))"`
**Explanation:** This command imports the botocore package and lists the `data/` subdirectory contents, which contains one folder per registered AWS service, confirming which services botocore knows about before issuing any API calls.

### Check which credentials botocore will use for the next API call
**Args:** `python -c "import botocore.session; s = botocore.session.get_session(); print(s.get_credentials().access_key)"`
**Explanation:** This retrieves the active botocore session, resolves the credential chain (env vars, config file, or instance profile), and prints the Access Key ID so you can verify credentials are loaded before a long operation.

### Inspect the endpoint URL that botocore will resolve for a specific service and region
**Args:** `python -c "import botocore.session; s = botocore.session.get_session(); r = s.create_client('s3', region_name='us-west-2'); print(r._endpoint)'"`
**Explanation:** Creating a botocore client does not make an HTTP request. This command instantiates the client and prints the resolved endpoint URL, useful for debugging routing issues before issuing `list_buckets` or `put_object`.

### Simulate how botocore signs a GET request to the STS GetCallerIdentity operation
**Args:** `python -c "import botocore.session; import json; s = botocore.session.get_session(); op = s.create_client('sts').meta.operation_model; print(json.dumps(op.service_model.operation_names, indent=2))"`
**Explanation:** This lists all operation names in the STS service model, not executing a request. It shows the exact names botocore expects in the JSON definition, helpful for scripting against the low-level API without relying on boto3's higher-level abstractions.

### Dump the raw JSON service definition for EC2 to inspect pagination models
**Args:** `python -c "import botocore.session; import os; path = os.path.join(botocore.session.get_session().get_service_model('ec2').__class__.__module__, 'ec2', '2016-11-15', 'service-2.json'); print(path)"`
**Explanation:** This constructs the filesystem path to the EC2 service definition JSON that botocore reads at runtime, allowing you to inspect paginator definitions and response shapes without issuing a live API call.