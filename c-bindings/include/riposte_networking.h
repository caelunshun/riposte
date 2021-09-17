#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


enum class RipError {
  None,
  Io,
};

/// Handle to a connected peer.
///
/// The connection runs over any IO system, such as
/// a QUIC endpoint or stdout/stdin. Messages
/// are length-prefixed.
struct RipConnectionHandle;

struct RipHubServerConnection;

/// A networking context. Internally
/// stores the Tokio runtime instance.
///
/// Maintains a list of callbacks, one for each
/// request performed.
struct RipNetworkingContext;

struct RipRasterizedMask;

struct RipResult;

/// Parameter 1: userdata passed into request
/// Parameter 2: result of request
using Callback = void(*)(void*, const RipResult*);

struct RipBytes {
  size_t len;
  const uint8_t *ptr;
};


extern "C" {

void hubconn_get_new_connection(RipNetworkingContext *ctx,
                                RipHubServerConnection *conn,
                                Callback callback,
                                void *userdata);

/// Frees a connection, disconnecting it.
void networkctx_conn_free(RipNetworkingContext *_ctx, RipConnectionHandle *conn);

/// Creates a request that receives data from the given connection.
///
/// `callback` is invoked after the data is received. The `data` field
/// contains a `bytes` variant if successful.
void networkctx_conn_recv_data(RipNetworkingContext *ctx,
                               RipConnectionHandle *conn,
                               Callback callback,
                               void *userdata);

/// Creates a request that sends data to the given connection.
///
/// `callback` is invoked after the data is sent. The `data` field
/// is always set to `None`.
///
/// `data` is copied; we don't take ownership.
void networkctx_conn_send_data(RipNetworkingContext *ctx,
                               const RipConnectionHandle *conn,
                               RipBytes data,
                               Callback callback,
                               void *userdata);

/// Creates a new connection handle operating on stdout/stdin.
///
/// Used in singleplayer mode when the server runs as a child process
/// of the client.
RipConnectionHandle *networkctx_connect_stdio(RipNetworkingContext *ctx);

/// Creates a new networking context.
RipNetworkingContext *networkctx_create();

RipHubServerConnection *networkctx_create_game(RipNetworkingContext *ctx,
                                               const uint8_t *host_auth_token,
                                               size_t host_auth_token_len);

/// Frees a networking context.
void networkctx_free(RipNetworkingContext *ctx);

/// Waits for one completed request and invokes its
/// callback.
///
/// Should be called in a loop to process more than one request.
void networkctx_wait(RipNetworkingContext *ctx);

RipBytes rip_result_get_bytes(const RipResult *res);

RipConnectionHandle *rip_result_get_connection(const RipResult *res);

const char *rip_result_get_connection_uuid(const RipResult *res);

RipError rip_result_get_error(const RipResult *res);

bool rip_result_is_success(const RipResult *res);

void zeno_mask_free(RipRasterizedMask *mask);

uint32_t zeno_mask_get_height(const RipRasterizedMask *mask);

uint8_t zeno_mask_get_value(const RipRasterizedMask *mask, uint32_t x, uint32_t y);

uint32_t zeno_mask_get_width(const RipRasterizedMask *mask);

/// Rasterizes the given line mesh into an alpha grid.
RipRasterizedMask *zeno_rasterize_lines(const float *coordinates, size_t num_points);

} // extern "C"
