/**
 * @fileoverview gRPC-Web generated client stub for moni
 * @enhanceable
 * @public
 */

// Code generated by protoc-gen-grpc-web. DO NOT EDIT.
// versions:
// 	protoc-gen-grpc-web v1.4.1
// 	protoc              v3.21.12
// source: proto/moni-rpc.proto


/* eslint-disable */
// @ts-nocheck



const grpc = {};
grpc.web = require('grpc-web');

const proto = {};
proto.moni = require('./moni-rpc_pb.js');

/**
 * @param {string} hostname
 * @param {?Object} credentials
 * @param {?grpc.web.ClientOptions} options
 * @constructor
 * @struct
 * @final
 */
proto.moni.MoniRpcServiceClient =
    function(hostname, credentials, options) {
  if (!options) options = {};
  options.format = 'binary';

  /**
   * @private @const {!grpc.web.GrpcWebClientBase} The client
   */
  this.client_ = new grpc.web.GrpcWebClientBase(options);

  /**
   * @private @const {string} The hostname
   */
  this.hostname_ = hostname.replace(/\/+$/, '');

};


/**
 * @param {string} hostname
 * @param {?Object} credentials
 * @param {?grpc.web.ClientOptions} options
 * @constructor
 * @struct
 * @final
 */
proto.moni.MoniRpcServicePromiseClient =
    function(hostname, credentials, options) {
  if (!options) options = {};
  options.format = 'binary';

  /**
   * @private @const {!grpc.web.GrpcWebClientBase} The client
   */
  this.client_ = new grpc.web.GrpcWebClientBase(options);

  /**
   * @private @const {string} The hostname
   */
  this.hostname_ = hostname.replace(/\/+$/, '');

};


/**
 * @const
 * @type {!grpc.web.MethodDescriptor<
 *   !proto.moni.LoginEmailRequest,
 *   !proto.moni.LoginEmailResponse>}
 */
const methodDescriptor_MoniRpcService_LoginEmail = new grpc.web.MethodDescriptor(
  '/moni.MoniRpcService/LoginEmail',
  grpc.web.MethodType.UNARY,
  proto.moni.LoginEmailRequest,
  proto.moni.LoginEmailResponse,
  /**
   * @param {!proto.moni.LoginEmailRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.moni.LoginEmailResponse.deserializeBinary
);


/**
 * @param {!proto.moni.LoginEmailRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @param {function(?grpc.web.RpcError, ?proto.moni.LoginEmailResponse)}
 *     callback The callback function(error, response)
 * @return {!grpc.web.ClientReadableStream<!proto.moni.LoginEmailResponse>|undefined}
 *     The XHR Node Readable Stream
 */
proto.moni.MoniRpcServiceClient.prototype.loginEmail =
    function(request, metadata, callback) {
  return this.client_.rpcCall(this.hostname_ +
      '/moni.MoniRpcService/LoginEmail',
      request,
      metadata || {},
      methodDescriptor_MoniRpcService_LoginEmail,
      callback);
};


/**
 * @param {!proto.moni.LoginEmailRequest} request The
 *     request proto
 * @param {?Object<string, string>=} metadata User defined
 *     call metadata
 * @return {!Promise<!proto.moni.LoginEmailResponse>}
 *     Promise that resolves to the response
 */
proto.moni.MoniRpcServicePromiseClient.prototype.loginEmail =
    function(request, metadata) {
  return this.client_.unaryCall(this.hostname_ +
      '/moni.MoniRpcService/LoginEmail',
      request,
      metadata || {},
      methodDescriptor_MoniRpcService_LoginEmail);
};


/**
 * @const
 * @type {!grpc.web.MethodDescriptor<
 *   !proto.moni.CreateProjectRequest,
 *   !proto.moni.CreateProjectResponse>}
 */
const methodDescriptor_MoniRpcService_CreateProject = new grpc.web.MethodDescriptor(
  '/moni.MoniRpcService/CreateProject',
  grpc.web.MethodType.UNARY,
  proto.moni.CreateProjectRequest,
  proto.moni.CreateProjectResponse,
  /**
   * @param {!proto.moni.CreateProjectRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.moni.CreateProjectResponse.deserializeBinary
);


/**
 * @param {!proto.moni.CreateProjectRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @param {function(?grpc.web.RpcError, ?proto.moni.CreateProjectResponse)}
 *     callback The callback function(error, response)
 * @return {!grpc.web.ClientReadableStream<!proto.moni.CreateProjectResponse>|undefined}
 *     The XHR Node Readable Stream
 */
proto.moni.MoniRpcServiceClient.prototype.createProject =
    function(request, metadata, callback) {
  return this.client_.rpcCall(this.hostname_ +
      '/moni.MoniRpcService/CreateProject',
      request,
      metadata || {},
      methodDescriptor_MoniRpcService_CreateProject,
      callback);
};


/**
 * @param {!proto.moni.CreateProjectRequest} request The
 *     request proto
 * @param {?Object<string, string>=} metadata User defined
 *     call metadata
 * @return {!Promise<!proto.moni.CreateProjectResponse>}
 *     Promise that resolves to the response
 */
proto.moni.MoniRpcServicePromiseClient.prototype.createProject =
    function(request, metadata) {
  return this.client_.unaryCall(this.hostname_ +
      '/moni.MoniRpcService/CreateProject',
      request,
      metadata || {},
      methodDescriptor_MoniRpcService_CreateProject);
};


module.exports = proto.moni;

