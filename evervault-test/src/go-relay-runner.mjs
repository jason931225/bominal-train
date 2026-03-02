import { spawn } from "node:child_process";

function normalizeText(value) {
  return String(value || "").trim();
}

export async function dispatchViaGoOutboundRelay({
  binaryPath,
  appId,
  apiKey,
  destinationUrl,
  method = "POST",
  headers = {},
  body = {},
  timeoutMs = 20000,
}) {
  const resolvedBinary = normalizeText(binaryPath);
  if (!resolvedBinary) {
    throw new Error("Go relay sender binary path is required");
  }

  const input = {
    app_uuid: normalizeText(appId),
    api_key: normalizeText(apiKey),
    destination_url: normalizeText(destinationUrl),
    method: String(method || "POST").toUpperCase(),
    headers: Object.fromEntries(
      Object.entries(headers).map(([key, value]) => [String(key), String(value ?? "")]),
    ),
    body,
    timeout_ms: Math.max(1000, Number(timeoutMs) || 20000),
  };

  if (!input.app_uuid || !input.api_key || !input.destination_url) {
    throw new Error("Missing required outbound relay inputs");
  }

  return new Promise((resolve, reject) => {
    const proc = spawn(resolvedBinary, [], {
      stdio: ["pipe", "pipe", "pipe"],
    });

    let stdout = "";
    let stderr = "";
    let settled = false;

    const timeout = setTimeout(() => {
      if (settled) return;
      settled = true;
      proc.kill("SIGKILL");
      reject(new Error("Go relay sender timed out"));
    }, input.timeout_ms + 1500);

    const finalize = (handler) => (value) => {
      if (settled) return;
      settled = true;
      clearTimeout(timeout);
      handler(value);
    };

    proc.stdout.setEncoding("utf8");
    proc.stderr.setEncoding("utf8");
    proc.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    proc.stderr.on("data", (chunk) => {
      stderr += chunk;
    });

    proc.on(
      "error",
      finalize((error) => {
        reject(new Error(`Failed to spawn Go relay sender: ${error.message}`));
      }),
    );

    proc.on(
      "close",
      finalize((code) => {
        if (code !== 0) {
          reject(new Error(`Go relay sender failed (exit ${code}): ${normalizeText(stderr) || normalizeText(stdout)}`));
          return;
        }

        const trimmed = normalizeText(stdout);
        if (!trimmed) {
          reject(new Error("Go relay sender returned empty output"));
          return;
        }

        try {
          const parsed = JSON.parse(trimmed);
          resolve(parsed);
        } catch (error) {
          reject(new Error(`Go relay sender returned invalid JSON: ${error.message}`));
        }
      }),
    );

    proc.stdin.end(JSON.stringify(input));
  });
}
