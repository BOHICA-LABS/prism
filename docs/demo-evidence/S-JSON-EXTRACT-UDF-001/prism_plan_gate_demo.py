#!/usr/bin/env python3
"""Plan-gate rejection demo for S-JSON-EXTRACT-UDF-001."""
import subprocess
import json
import os
import sys

PRISM = "/Users/jmagady/Dev/prism/.worktrees/S-JSON-EXTRACT-UDF-001/target/debug/prism"

def run_query(query, label):
    env = os.environ.copy()
    env["PRISM_CONFIG_DIR"] = "/tmp/prism-demo-config"
    env["CLAROTY_INSTANCE_URL"] = "https://demo.claroty.example.com"
    env["PRISM_DISABLE_PLUGIN_LOAD"] = "1"
    
    msgs = [
        {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"demo","version":"1.0"}}},
        {"jsonrpc":"2.0","method":"notifications/initialized","params":{}},
        {"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"query","arguments":{"query":query,"clients":["acme"]}}},
    ]
    inp = "\n".join(json.dumps(m) for m in msgs) + "\n"
    p = subprocess.Popen([PRISM, "start"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)
    try:
        out, _ = p.communicate(inp.encode(), timeout=10)
    except subprocess.TimeoutExpired:
        p.kill()
        out, _ = p.communicate()
    
    print(f"\n>>> {label}")
    print(f"    Query: {query[:100]}{'...' if len(query)>100 else ''}")
    
    for line in out.decode().split("\n"):
        if line.strip():
            try:
                obj = json.loads(line)
                if obj.get("id") == 2:
                    r = obj["result"]
                    sc = r.get("structuredContent", {}).get("error", {})
                    if sc:
                        print(f"  isError: True")
                        print(f"  code: {sc.get('code', 'N/A')}")
                        print(f"  message: {sc.get('message', 'N/A')[:120]}")
                    else:
                        print("  isError: False — query accepted, passed plan gate")
                        if r.get("content"):
                            print(f"  content: {r['content'][0]['text'][:100]}")
            except Exception as e:
                pass

key_257 = "a" * 257
key_256 = "a" * 256

print("=== S-JSON-EXTRACT-UDF-001: json_extract_string Plan-Gate Demo ===")

run_query(
    "SELECT json_extract_string(raw_extensions, severity_id) FROM claroty_alerts",
    "AC-006: Non-literal key -> E-QUERY-045(a)"
)

run_query(
    f"SELECT json_extract_string(raw_extensions, '{key_257}') FROM claroty_alerts",
    "AC-007: 257-byte key -> E-QUERY-045(b)"
)

run_query(
    f"SELECT json_extract_string(raw_extensions, '{key_256}') FROM claroty_alerts",
    "AC-007 boundary: 256-byte key -> ACCEPTED"
)

run_query(
    "SELECT id FROM claroty_alerts WHERE json_extract_string(raw_extensions, severity_id) = 'x'",
    "AC-012: WHERE predicate non-literal -> E-QUERY-045(a)"
)

run_query(
    "SELECT id FROM claroty_alerts WHERE json_extract_string(raw_extensions, 'severity') = 'high'",
    "AC-012: WHERE predicate valid literal -> ACCEPTED"
)

run_query(
    "SELECT json_extract_string(raw_extensions, 'severity') AS jex_severity FROM claroty_alerts | limit 5",
    "AC-011: SQL-pipe mode UDF resolves (no 'unknown function')"
)

print("\n=== All plan-gate demos complete ===")
