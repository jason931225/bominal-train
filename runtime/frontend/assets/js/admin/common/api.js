export const requestJson = async (url, method = "GET", payload = null) => {
  const options = {
    method,
    headers: {
      Accept: "application/json",
    },
  };
  if (payload !== null) {
    options.headers["Content-Type"] = "application/json";
    options.body = JSON.stringify(payload);
  }

  let response;
  try {
    response = await fetch(url, options);
  } catch (error) {
    return {
      ok: false,
      status: 0,
      body: { message: String(error), request_id: "n/a" },
    };
  }

  let body = null;
  try {
    body = await response.json();
  } catch (_error) {
    body = null;
  }

  return { ok: response.ok, status: response.status, body };
};

export const errorMessage = (result) => {
  const body = result && result.body ? result.body : {};
  const message = body.message || "Request failed";
  const requestId = body.request_id ? ` (request_id: ${body.request_id})` : "";
  return `${message}${requestId}`;
};
