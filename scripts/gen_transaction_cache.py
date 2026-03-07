#!/usr/bin/env python3
"""Generate transaction cache for x-cli using heimdall's ClientTransaction."""
import json
import sys
import os
import time
import re

# Clear proxy env vars
for k in ['http_proxy', 'https_proxy', 'HTTP_PROXY', 'HTTPS_PROXY', 'all_proxy', 'ALL_PROXY']:
    os.environ.pop(k, None)

sys.path.insert(0, os.path.expanduser("~/PycharmProjects/heimdall"))

import requests
import base64
import bs4

# Patterns from heimdall's x_client_transaction.py
ON_DEMAND_FILE_REGEX = re.compile(r"""['"]ondemand\.s['"]\s*:\s*['"](\w*)['"]""")
INDICES_REGEX = re.compile(r"""\(\w\[(\d{1,2})\],\s*16\)""")


def fetch_home_page():
    """Fetch x.com homepage without proxy."""
    session = requests.Session()
    session.trust_env = False  # no proxy
    for attempt in range(10):
        try:
            resp = session.get("https://x.com", headers={
                "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36"
            }, timeout=15)
            return bs4.BeautifulSoup(resp.text, "html.parser"), resp.text
        except Exception as e:
            if attempt == 9:
                raise
            time.sleep(0.5)


def main():
    soup, home_html = fetch_home_page()

    # Extract on-demand JS hash
    match = ON_DEMAND_FILE_REGEX.search(home_html)
    if not match:
        raise RuntimeError("Could not find ondemand.s file reference")

    js_url = f"https://abs.twimg.com/responsive-web/client-web/ondemand.s.{match.group(1)}a.js"
    session = requests.Session()
    session.trust_env = False
    js_resp = session.get(js_url, timeout=15)
    indices = [int(m.group(1)) for m in INDICES_REGEX.finditer(js_resp.text)]

    if not indices:
        raise RuntimeError("Could not extract key byte indices from JS")

    row_index = indices[0]
    key_byte_indices = indices[1:]

    # Extract verification key
    meta = soup.find("meta", attrs={"name": "twitter-site-verification"})
    if not meta:
        raise RuntimeError("Verification key not found")
    key_b64 = meta["content"]
    key_bytes = list(base64.b64decode(key_b64))

    # Compute animation key (using heimdall's implementation)
    sys.path.insert(0, os.path.expanduser("~/PycharmProjects/heimdall"))
    from twitter_crawler.utils.x_client_transaction import ClientTransaction

    # Create a dummy CT and manually set values
    ct = ClientTransaction.__new__(ClientTransaction)
    ct.home_page = soup
    ct.row_index = row_index
    ct.key_byte_indices = key_byte_indices
    ct.key = key_b64
    ct.key_bytes = key_bytes
    ct.animation_key = ct._compute_animation_key(key_bytes)

    cache = {
        "key_bytes": key_bytes,
        "animation_key": ct.animation_key,
        "created_at": int(time.time()),
    }

    cache_dir = os.path.expanduser("~/.x-cli")
    os.makedirs(cache_dir, exist_ok=True)
    cache_path = os.path.join(cache_dir, "transaction_cache.json")

    with open(cache_path, "w") as f:
        json.dump(cache, f)

    print(f"Cache written to {cache_path}")
    print(f"key_bytes length: {len(key_bytes)}")
    print(f"animation_key: {ct.animation_key}")


if __name__ == "__main__":
    main()
