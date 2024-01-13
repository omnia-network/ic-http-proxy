import IcWebSocket, { createWsConfig, generateRandomIdentity } from "ic-websocket-js";
import { proxy_canister, canisterId } from "./src/canister/declarations/proxy_canister";

/**
 * How long to wait before trying to reconnect
 * in case the WS connection with the canister was manually closed
 */
const RECONNECT_AFTER_MS = 45_000;

const icNetworkUrl = process.env.IC_NETWORK_URL as string;
const gatewayUrl = process.env.IC_WS_GATEWAY_URL as string;

const wsConfig = createWsConfig({
  canisterId,
  canisterActor: proxy_canister,
  networkUrl: icNetworkUrl,
  identity: generateRandomIdentity(),
});

console.log("Canister ID:", canisterId);

const openWsConnection = () => {
  const ws = new IcWebSocket(gatewayUrl, {}, wsConfig);
  const principal = ws.getPrincipal().toString();
  console.log("WebSocket principal:", principal);

  ws.onopen = () => {
    console.log("WebSocket connected with principal", principal);
  };

  ws.onmessage = async (ev) => {
    const incomingMessage = ev.data;
    // console.log("Message", incomingMessage);

    if ("HttpRequest" in incomingMessage) {
      const requestId = incomingMessage.HttpRequest[0];
      const request = incomingMessage.HttpRequest[1];

      const url = new URL(request.url);
      const method = Object.keys(request.method)[0]; // workaround to get the candid enum
      const headers = new Headers(
        request.headers.map(({ name, value }) => [name, value] as [string, string])
      );
      const body = (request.body.length > 0 && method !== "GET")
        ? new Uint8Array(request.body[0]!)
        : null;

      console.log(
        "\nExecuting HTTP request:",
        "\nurl:", url.toString(),
        "\nmethod:", method,
        "\nheaders:", headers,
        "\nbody bytes:", body?.length,
        "\nbody:", body ? new TextDecoder().decode(body) : null
      );

      try {
        const response = await fetch(url, {
          method,
          headers,
          body,
        });

        const responseBody = new Uint8Array(await response.arrayBuffer());

        console.log(
          "HTTP response:",
          "\nurl:", request.url,
          "\nstatus:", response.status,
          "\nbody bytes:", responseBody.byteLength,
          "\nbody:", new TextDecoder().decode(responseBody),
        );

        ws.send({
          HttpResponse: [
            requestId,
            {
              status: BigInt(response.status),
              headers: Array.from(response.headers.entries()).map(([key, value]) => ({
                name: key,
                value,
              })),
              body: responseBody,
            },
          ],
        });

        console.log("Sent response over WebSocket.");
      } catch (e) {
        console.error("http-over-ws: error", e);
        ws.send({
          Error: [[requestId], String(e)],
        });
      }
    } else if ("Error" in incomingMessage) {
      console.error("http-over-ws: incoming error:", incomingMessage.Error);
    }
  };

  ws.onclose = (ev) => {
    console.warn("WebSocket disconnected. Reason:", ev.reason);

    // if there are problems with the WebSocket itself, don't reconnect
    // this may occur also when connecting to a non-existing canister,
    // since the ws gateway can't relay the open message and closes the connection
    if (ev.reason === "Connection ended") {
      return;
    }

    // reconnect immediately if the connection was closed due to an error
    // otherwise wait for some time as it may be a canister upgrade
    // TODO: use better logic here
    const reconnectAfterMs = ev.reason === "ClosedByApplication" ? RECONNECT_AFTER_MS : 0;

    if (reconnectAfterMs > 0) {
      console.log("Reconnecting in", reconnectAfterMs / 1000, "seconds...");
    }

    setTimeout(() => {
      console.log("Reconnecting...");
      openWsConnection();
    }, reconnectAfterMs);
  };

  ws.onerror = (ev) => {
    console.error("WebSocket error:", ev.message);
  };
};

openWsConnection();
