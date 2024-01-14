import IcWebSocket, { createWsConfig, generateRandomIdentity } from "ic-websocket-js";
import { proxy_canister, canisterId } from "./src/canister/declarations/proxy_canister";
import { printVersion } from "./src/utils";

/**
 * How many seconds to wait before trying to reconnect
 * in case the WS connection with the canister was manually closed
 */
const RECONNECT_AFTER_SECONDS = Number(process.env.RECONNECT_AFTER_SECONDS) || 45;

const IC_NETWORK_URL = process.env.IC_NETWORK_URL as string;
const IC_WS_GATEWAY_URL = process.env.IC_WS_GATEWAY_URL as string;

console.log(`Config:
  IC_NETWORK_URL=${IC_NETWORK_URL},
  IC_WS_GATEWAY_URL=${IC_WS_GATEWAY_URL},
  RECONNECT_AFTER_SECONDS=${RECONNECT_AFTER_SECONDS}`
);
printVersion();

const wsConfig = createWsConfig({
  canisterId,
  canisterActor: proxy_canister,
  networkUrl: IC_NETWORK_URL,
  identity: generateRandomIdentity(),
});

console.log("Canister ID:", canisterId);

const openWsConnection = () => {
  const ws = new IcWebSocket(IC_WS_GATEWAY_URL, {}, wsConfig);
  const principal = ws.getPrincipal().toString();
  console.log("WebSocket principal:", principal);

  ws.onopen = () => {
    console.log("WebSocket connected with principal", principal);

    // send the setup message
    ws.send({
      SetupProxyClient: null,
    });

    console.log("Setup message sent");
  };

  ws.onmessage = async (ev) => {
    try {
      const incomingMessage = ev.data;
      // console.log("Message", incomingMessage);

      if ("HttpRequest" in incomingMessage) {
        const requestId = incomingMessage.HttpRequest[0];
        const request = incomingMessage.HttpRequest[1];

        try {
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
            "\nbody bytes:", body?.length || 0,
            // "\nbody:", body ? new TextDecoder().decode(body) : null
          );

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
            // "\nbody:", new TextDecoder().decode(responseBody),
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
          console.error("http-over-ws: error for request id:", requestId, e);
          ws.send({
            Error: [[requestId], String(e)],
          });
        }
      } else if ("Error" in incomingMessage) {
        console.error("http-over-ws: incoming error:", incomingMessage.Error);
      }
    } catch (e) {
      console.error("http-over-ws: error", e);
      ws.send({
        Error: [[], String(e)],
      });
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
    const reconnectAfterSecs = ev.reason === "ClosedByApplication" ? RECONNECT_AFTER_SECONDS : 0;

    if (reconnectAfterSecs > 0) {
      console.log("Reconnecting in", reconnectAfterSecs, "seconds...");
    }

    setTimeout(() => {
      console.log("Reconnecting...");
      openWsConnection();
    }, reconnectAfterSecs * 1000);
  };

  ws.onerror = (ev) => {
    console.error("WebSocket error:", ev.message);
  };
};

openWsConnection();
