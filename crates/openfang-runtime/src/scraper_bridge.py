#!/usr/bin/env python3
"""OpenFang Scraper Bridge — Scrapling-powered web extraction over JSON-line stdio.

Reads JSON commands from stdin (one per line), fetches URLs using Scrapling's
stealth and dynamic fetchers, and writes JSON responses to stdout (one per line).

Two fetcher modes:
  FetchUrl    — StealthyFetcher: fast HTTP with anti-bot headers, no JS rendering.
                Works for most sites. Falls back to raw requests if Scrapling missing.
  FetchDynamic — DynamicFetcher: full browser + Playwright, handles JS-heavy SPAs,
                Cloudflare Turnstile, and lazy-loaded content.

Usage:
    python scraper_bridge.py [--timeout 30]
"""

import argparse
import json
import sys
import re


def main():
    parser = argparse.ArgumentParser(description="OpenFang Scraper Bridge")
    parser.add_argument("--timeout", type=int, default=30)
    args = parser.parse_args()

    timeout = args.timeout

    # Signal ready immediately
    respond({"success": True, "data": {"status": "ready"}})

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            cmd = json.loads(line)
            action = cmd.get("action", "")
            result = handle_command(action, cmd, timeout)
            respond(result)
        except Exception as e:
            respond({"success": False, "error": f"{type(e).__name__}: {e}"})


def handle_command(action, cmd, timeout):
    url = cmd.get("url", "")
    if not url:
        return {"success": False, "error": "Missing 'url' parameter"}

    if action == "FetchUrl":
        return fetch_stealth(url, timeout)
    elif action == "FetchDynamic":
        wait_for = cmd.get("wait_for", None)
        return fetch_dynamic(url, timeout, wait_for)
    elif action == "Ping":
        return {"success": True, "data": {"pong": True}}
    else:
        return {"success": False, "error": f"Unknown action: {action}"}


def fetch_stealth(url, timeout):
    """Fetch with StealthyFetcher — anti-bot headers, no JS rendering."""
    try:
        from scrapling import StealthyFetcher
        fetcher = StealthyFetcher(auto_match=False)
        page = fetcher.fetch(url, timeout=timeout)
        content = extract_text(page)
        return {
            "success": True,
            "data": {
                "url": url,
                "status": page.status if hasattr(page, 'status') else 200,
                "content": content,
                "mode": "stealth",
            }
        }
    except ImportError:
        # Scrapling not installed — fall back to plain requests
        return fetch_plain(url, timeout)
    except Exception as e:
        # If stealth fetch fails, try plain
        try:
            return fetch_plain(url, timeout)
        except Exception as e2:
            return {"success": False, "error": f"StealthyFetcher failed: {e}. Fallback also failed: {e2}"}


def fetch_dynamic(url, timeout, wait_for=None):
    """Fetch with DynamicFetcher — full browser via Playwright, JS rendering."""
    try:
        from scrapling import DynamicFetcher
        fetcher = DynamicFetcher(auto_match=False)
        kwargs = {"timeout": timeout * 1000}  # Scrapling uses ms for dynamic
        if wait_for:
            kwargs["wait_selector"] = wait_for
        page = fetcher.fetch(url, **kwargs)
        content = extract_text(page)
        return {
            "success": True,
            "data": {
                "url": url,
                "content": content,
                "mode": "dynamic",
            }
        }
    except ImportError as e:
        if "playwright" in str(e).lower() or "scrapling" in str(e).lower():
            return {
                "success": False,
                "error": (
                    "scrape_dynamic requires Scrapling + Playwright. "
                    "Install with: pip install scrapling && playwright install chromium"
                )
            }
        return {"success": False, "error": f"Import error: {e}"}
    except Exception as e:
        return {"success": False, "error": f"DynamicFetcher failed: {e}"}


def fetch_plain(url, timeout):
    """Plain requests fallback — no anti-bot, but works when Scrapling unavailable."""
    import urllib.request
    headers = {
        "User-Agent": (
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
            "AppleWebKit/537.36 (KHTML, like Gecko) "
            "Chrome/120.0.0.0 Safari/537.36"
        ),
        "Accept": "text/html,application/xhtml+xml,*/*;q=0.8",
        "Accept-Language": "en-US,en;q=0.9",
    }
    req = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        body = resp.read().decode("utf-8", errors="replace")
    content = html_to_text(body)
    return {
        "success": True,
        "data": {
            "url": url,
            "status": resp.status if hasattr(resp, 'status') else 200,
            "content": content,
            "mode": "plain_fallback",
        }
    }


def extract_text(page):
    """Extract readable text from a Scrapling page object."""
    try:
        # Try Scrapling's built-in markdown/text extraction
        if hasattr(page, 'get_all_text'):
            text = page.get_all_text(ignore_tags=('script', 'style', 'nav', 'footer', 'header'))
        elif hasattr(page, 'body'):
            # Scrapling page has .body property with the HTML
            text = html_to_text(str(page.body) if page.body else "")
        elif hasattr(page, 'html_content'):
            text = html_to_text(page.html_content)
        else:
            text = str(page)
    except Exception:
        try:
            text = str(page)
        except Exception:
            text = "(could not extract content)"

    # Truncate to prevent huge payloads
    max_chars = 50_000
    if len(text) > max_chars:
        text = text[:max_chars] + f"\n\n[Truncated — {len(text)} total chars]"
    return text


def html_to_text(html):
    """Minimal HTML-to-text via regex — used as fallback when Scrapling unavailable."""
    # Remove script/style blocks
    html = re.sub(r'<(script|style)[^>]*>.*?</\1>', '', html, flags=re.DOTALL | re.IGNORECASE)
    # Replace headings with markdown
    html = re.sub(r'<h([1-6])[^>]*>(.*?)</h\1>', lambda m: '\n' + '#' * int(m.group(1)) + ' ' + m.group(2) + '\n', html, flags=re.DOTALL | re.IGNORECASE)
    # Replace list items
    html = re.sub(r'<li[^>]*>(.*?)</li>', r'\n- \1', html, flags=re.DOTALL | re.IGNORECASE)
    # Replace paragraphs and divs with newlines
    html = re.sub(r'<(p|div|br|tr)[^>]*>', '\n', html, flags=re.IGNORECASE)
    # Remove remaining tags
    html = re.sub(r'<[^>]+>', '', html)
    # Decode common entities
    html = html.replace('&amp;', '&').replace('&lt;', '<').replace('&gt;', '>').replace('&nbsp;', ' ').replace('&#39;', "'").replace('&quot;', '"')
    # Collapse whitespace
    html = re.sub(r'\n{3,}', '\n\n', html)
    html = re.sub(r'[ \t]+', ' ', html)
    return html.strip()


def respond(data):
    """Write a JSON response line to stdout."""
    sys.stdout.write(json.dumps(data) + "\n")
    sys.stdout.flush()


if __name__ == "__main__":
    main()
