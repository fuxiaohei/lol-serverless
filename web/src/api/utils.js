import { getLocalUser } from "./login.js";

const { RpcServiceClient } = require("./proto/land-rpc_grpc_web_pb.js");

const RPC_CLIENT_ADDRESS = window.API_ADDRESS || "http://127.0.0.1:38779";

function createClient() {
  let client = new RpcServiceClient(RPC_CLIENT_ADDRESS);
  return client;
}

async function callClient(request, callFunc) {
  let client = createClient();
  let promise = new Promise((resolve, reject) => {
    let fn = client[callFunc];
    if (!fn) {
      reject("no such function");
      return;
    }
    console.log("callClient:" + callFunc + ",request:", request.toObject());
    let metadata = {
      "x-grpc-method": String(callFunc),
    };
    let user = getLocalUser();
    if (user && user.accessToken) {
      metadata["Authorization"] = "Bearer " + user.accessToken;
    }
    client[callFunc](request, metadata, (err, response) => {
      if (err) {
        console.log("callClient:" + callFunc + ",error:", err);
        resolve({ error: String(err) });
        return;
      }
      console.log("callClient:" + callFunc + ",response:", response.toObject());
      resolve(response.toObject());
    });
  });
  return promise;
}

console.log("RPC_CLIENT_ADDRESS:", RPC_CLIENT_ADDRESS);

export { RPC_CLIENT_ADDRESS, callClient };
